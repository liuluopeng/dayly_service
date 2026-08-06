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
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
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
        .nest(
            "/api/melatonin",
            crate::controller::melatonin::melatonin_routes(),
        )
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
        .nest(
            "/api/admin/user-directories",
            crate::controller::user_directories::admin_user_dir_routes(),
        )
        // 媒体路径管理
        .nest(
            "/api/media_paths",
            crate::controller::media_paths::media_paths_routes(),
        )
        // 聊天路由
        .nest("/api/chat", crate::controller::chat::chat_routes())
        // WebRTC 共享路由
        .nest("/api/webrtc", crate::controller::webrtc::webrtc_routes())
        // 剪贴板相关路由
        .nest(
            "/api/clipboard",
            crate::controller::clipboard::clipboard_routes(),
        )
        // 添加认证中间件
        .layer(axum_middleware::from_fn(auth_middleware));

    let mut router = Router::new()
        .route(
            "/graphql",
            get(crate::graphql::graphql_playground).post(crate::graphql::graphql),
        )
        .layer(Extension(schema))
        .nest("/api/user", crate::controller::user_routes())
        .nest("/api/zici", crate::controller::zici::zici_routes())
        .merge(secured_routes)
        .merge(crate::controller::dict_resource_routes())
        .nest("/api/pinyin", crate::controller::pinyin::pinyin_routes())
        .merge(crate::controller::songs::songs_cover_route())
        .merge(crate::controller::songs::songs_file_route())
        .route("/", get(root_index))
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
        .route("/vue", get(|| async { Redirect::permanent("/vue/") }))
        .nest_service(
            "/vue/",
            ServeDir::new("static/vue/")
                .append_index_html_on_directories(true)
                .precompressed_gzip(),
        )
        .route("/wasm", get(|| async { Redirect::permanent("/wasm/") }))
        .nest_service(
            "/wasm/",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    HeaderName::from_static("cross-origin-opener-policy"),
                    HeaderValue::from_static("same-origin"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    HeaderName::from_static("cross-origin-embedder-policy"),
                    HeaderValue::from_static("credentialless"),
                ))
                .service(
                    ServeDir::new("static/wasm/")
                        .append_index_html_on_directories(true)
                        .precompressed_gzip(),
                ),
        )
        .route(
            "/flutter",
            get(|| async { Redirect::permanent("/flutter/") }),
        )
        .nest_service(
            "/flutter/",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    HeaderName::from_static("cross-origin-opener-policy"),
                    HeaderValue::from_static("same-origin"),
                ))
                .layer(SetResponseHeaderLayer::overriding(
                    HeaderName::from_static("cross-origin-embedder-policy"),
                    HeaderValue::from_static("credentialless"),
                ))
                .service(
                    ServeDir::new("static/flutter/")
                        .append_index_html_on_directories(true)
                        .precompressed_gzip(),
                ),
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
                    let uri = req.uri();
                    let uri_str = if let Some(query) = uri.query() {
                        if query.contains("token=") {
                            let redacted_query = query
                                .split('&')
                                .map(|pair| {
                                    if let Some((key, _)) = pair.split_once('=') {
                                        if key == "token" {
                                            "token=<redacted>".to_string()
                                        } else {
                                            pair.to_string()
                                        }
                                    } else {
                                        pair.to_string()
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("&");
                            format!("{}?{}", uri.path(), redacted_query)
                        } else {
                            uri.to_string()
                        }
                    } else {
                        uri.to_string()
                    };
                    info_span!(
                        "http",
                        method = %req.method(),
                        uri = %uri_str,
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

    router = router
        .nest(
            "/api/mhtml",
            crate::controller::mhtml_convert::mhtml_routes(),
        )
        .nest("/api", crate::controller::ocr::ocr_routes());

    router
}

pub async fn root_index() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html lang="zh">
<head><meta charset="utf-8"><title>Dayly Service</title>
<style>
body{font-family:system-ui,sans-serif;display:flex;justify-content:center;align-items:center;min-height:100vh;margin:0;background:#1e1e1e;color:#ccc}
.card{background:#2d2d2d;border-radius:12px;padding:48px;text-align:center}
h1{margin:0 0 32px;color:#fff}
.links{display:flex;gap:16px;justify-content:center;flex-wrap:wrap}
.links a{display:block;padding:16px 32px;border-radius:8px;text-decoration:none;color:#fff;font-size:18px;transition:transform .15s,opacity .15s}
.links a:hover{transform:translateY(-2px);opacity:.9}
.vue{background:#4fc08d}
.wasm{background:#654ff0}
.flutter{background:#02569b}
</style></head>
<body><div class="card">
<h1>Dayly Service</h1>
<div class="links">
<a class="vue" href="/vue/">Vue 前端</a>
<a class="wasm" href="/wasm/">WASM 演示</a>
<a class="flutter" href="/flutter/">Flutter Web</a>
</div></div></body></html>"#,
    )
}

pub async fn http_logging_middleware(
    req: Request<Body>,
    next: axum_middleware::Next,
) -> Response<Body> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();

    let headers_str = headers
        .iter()
        .map(|(name, value): (&HeaderName, &HeaderValue)| {
            let value_str = if name.as_str().eq_ignore_ascii_case("authorization")
                || name.as_str().eq_ignore_ascii_case("cookie")
                || name.as_str().eq_ignore_ascii_case("proxy-authorization")
            {
                "<redacted>".to_string()
            } else {
                value.to_str().unwrap_or("<invalid>").to_string()
            };
            format!("{}: {}", name.as_str(), value_str)
        })
        .collect::<Vec<_>>()
        .join(", ");

    // 脱敏 URI 中的 ?token= 查询参数
    let uri_str = if let Some(query) = uri.query() {
        if query.contains("token=") {
            let redacted_query = query
                .split('&')
                .map(|pair| {
                    if let Some((key, _)) = pair.split_once('=') {
                        if key == "token" {
                            "token=<redacted>".to_string()
                        } else {
                            pair.to_string()
                        }
                    } else {
                        pair.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", uri.path(), redacted_query)
        } else {
            uri.to_string()
        }
    } else {
        uri.to_string()
    };

    tracing::info!(
        "HTTP 请求: method={}, path={}, headers=[{}]",
        method,
        uri_str,
        headers_str
    );

    let start_time = std::time::Instant::now();
    let mut response = next.run(req).await;
    let duration = start_time.elapsed();
    let status = response.status();
    let status_code = status.as_u16();

    if status_code >= 400 {
        let (parts, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .unwrap_or_default();
        let body_str = std::str::from_utf8(&body_bytes).unwrap_or("<non-utf8>");
        tracing::error!(
            "HTTP 错误: method={}, path={}, status={}, duration={:?}, body={}",
            method,
            uri_str,
            status_code,
            duration,
            body_str
        );
        response = Response::from_parts(parts, axum::body::Body::from(body_bytes));
    } else {
        tracing::info!(
            "HTTP 响应: method={}, path={}, status={}, duration={:?}",
            method,
            uri_str,
            status_code,
            duration
        );
    }

    response
}
