use crate::config::{ServerConfig, env::OpenAiConfig};
use crate::graphql::{AppSchema, MutationRoot, QueryRoot};
use crate::middleware::{JwtSecret, auth_middleware};
use axum::Extension;
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, Request, Response};
use axum::response::Redirect;
use axum::{Router, middleware as axum_middleware, routing::get};
use clap::Parser;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::{Span, info_span};

pub fn create_app(
    pg_pool: PgPool,
    jwt_secret: String,
    server_config: ServerConfig,
    redis_conn: ConnectionManager,
    chat_tx: broadcast::Sender<String>,
    signaling_state: crate::controller::webrtc::SignalingState,
) -> Router {
    let schema = AppSchema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(pg_pool.clone())
        .finish();

    // 加载 OpenAI 配置
    let openai_config = OpenAiConfig::parse();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(vec![
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
            HeaderName::from_static("token"),
        ])
        .expose_headers(vec![
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ]);

    // 创建需要认证的路由组
    let secured_routes = Router::new()
        // 歌曲相关路由
        .nest("/api/songs", crate::controller::songs::songs_routes())
        // GGTT相关路由
        .nest("/api/ggtt", crate::controller::ggtt::ggtt_routers())
        // melatonin相关路由
        .nest("/api/melatonin", crate::controller::melatonin::melatonin_routes())
        // ShortNotes相关路由
        .nest(
            "/api/short_notes",
            crate::controller::short_notes::short_notes_routes(),
        )
        // 笔记相关路由
        .nest("/api/note", crate::controller::note::note_routes())
        // OpenAI 相关路由
        .nest("/api/openai", crate::controller::openai::openai_routes())
        .nest(
            "/api/openai/sessions",
            crate::controller::openai_session::openai_session_routes(),
        )
        // 词典相关路由
        .nest("/api/dict", crate::controller::dict::dict_routes())
        // 文件服务路由
        .nest("/api/files", crate::controller::files::file_routes())
        // 图片相关路由
        .nest("/api/images", crate::controller::images::images_routes())
        // 视频相关路由
        .nest("/api/videos", crate::controller::videos::videos_routes())
        // 用户管理路由（需要认证）
        .nest("/api/user", crate::controller::user::secured_user_routes())
        // 管理员：用户目录管理
        .nest("/api/admin/user-directories", crate::controller::user_directories::admin_user_dir_routes())
        // 媒体路径管理
        .nest("/api/media_paths", crate::controller::media_paths::media_paths_routes())
        // 聊天路由
        .nest("/api/chat", crate::controller::chat::chat_routes())
        // WebRTC 共享路由
        .nest("/api/webrtc", crate::controller::webrtc::webrtc_routes())
        // 添加认证中间件
        .layer(axum_middleware::from_fn(auth_middleware));

    let mut router = Router::new()
        .route(
            "/graphql",
            get(crate::graphql::graphql_playground).post(crate::graphql::graphql),
        )
        .layer(Extension(schema))
        .nest("/api/user", crate::controller::user_routes())
        .merge(secured_routes)
        .merge(crate::controller::dict_resource_routes())
        .nest("/api/pinyin", crate::controller::pinyin::pinyin_routes())
        .merge(crate::controller::songs::songs_cover_route())
        .merge(crate::controller::songs::songs_file_route())
        .route("/hello", get(crate::handlers::hello_world))
        .route("/hi", get(|| async { "Hello, World!" }))
        .nest_service("/cover/", ServeDir::new("cover/").precompressed_gzip())
        .nest_service(
            "/whitenoise/",
            ServeDir::new("static/whitenoise/").precompressed_gzip(),
        )
        .nest_service(
            "/site/",
            ServeDir::new("static/hello/")
                .append_index_html_on_directories(true)
                .precompressed_gzip(),
        )
        .route("/dist", get(|| async { Redirect::permanent("/dist/") }))
        .nest_service(
            "/dist/",
            ServeDir::new("static/dist/")
                .append_index_html_on_directories(true)
                .precompressed_gzip(),
        )
        .layer(cors.clone())
        .layer(Extension(pg_pool))
        .layer(Extension(JwtSecret(jwt_secret)))
        .layer(Extension(server_config))
        .layer(Extension(openai_config))
        .layer(Extension(redis_conn))
        .layer(Extension(chat_tx))
        .layer(Extension(signaling_state))
        .layer(axum_middleware::from_fn(http_logging_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    info_span!(
                        "http",
                        method = %req.method(),
                        uri = %req.uri(),
                        status = tracing::field::Empty,
                        latency_ms = tracing::field::Empty,
                    )
                })
                .on_response(
                    |res: &Response<_>, latency: std::time::Duration, span: &Span| {
                        span.record("status", res.status().as_u16());
                        span.record("latency_ms", latency.as_millis());
                    },
                ),
        );

    // 使用fallback机制处理所有未匹配的路由，指向static/hello目录
    // 这样既能保持API路由正常工作，又能方便地访问所有子目录
    router = router
        .nest("/api/mhtml", crate::controller::mhtml_convert::mhtml_routes())
        .nest("/api", crate::controller::ocr::ocr_routes());
    router = router.fallback_service(
        ServeDir::new("static/hello/")
            .append_index_html_on_directories(true)
            .precompressed_gzip(),
    );

    router
}

pub async fn http_logging_middleware(
    req: Request<Body>,
    next: axum_middleware::Next,
) -> Response<Body> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    let path = uri.path();

    let headers_str = headers
        .iter()
        .map(|(name, value): (&HeaderName, &HeaderValue)| {
            format!(
                "{}: {}",
                name.as_str(),
                value.to_str().unwrap_or("<invalid>")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    tracing::info!(
        "HTTP 请求: method={}, path={}, headers=[{}]",
        method,
        path,
        headers_str
    );

    let start_time = std::time::Instant::now();
    let response = next.run(req).await;
    let duration = start_time.elapsed();
    let status = response.status();

    // 根据状态码添加颜色，使用不同的格式
    let status_code = status.as_u16();
    let colored_status = match status_code / 100 {
        1 => format!("\u{001b}[34m{}\u{001b}[0m", status_code), // 1xx - 蓝色
        2 => format!("\u{001b}[32m{}\u{001b}[0m", status_code), // 2xx - 绿色
        3 => format!("\u{001b}[33m{}\u{001b}[0m", status_code), // 3xx - 黄色
        4 => format!("\u{001b}[31m{}\u{001b}[0m", status_code), // 4xx - 红色
        5 => format!("\u{001b}[31m{}\u{001b}[0m", status_code), // 5xx - 红色
        _ => status_code.to_string(),                           // 其他 - 默认
    };

    tracing::info!(
        "HTTP 响应: method={}, path={}, status={}, duration={:?}",
        method,
        path,
        colored_status,
        duration
    );

    response
}
