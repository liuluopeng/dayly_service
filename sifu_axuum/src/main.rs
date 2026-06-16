use clap::Parser;
use lx_dayly_service::config::env::{RedisConfig, ServerConfig};
use sqlx::postgres::PgPoolOptions;
use tracing::info;

#[tokio::main]
async fn main() {
    // .env 在 sifu_axuum/ 下，cargo run 从 workspace root 启动时优先找它
    dotenv::from_filename(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env")).ok();
    dotenv::dotenv().ok();
    let config = lx_dayly_service::config::env::PgConfig::parse();
    let uri = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.pg_user, config.pg_password, config.pg_host, config.pg_port, config.pg_database
    );

    let pg_pool = PgPoolOptions::new()
        .connect(&uri)
        .await
        .expect("数据库连接失败");

    // redis
    let redis_config = RedisConfig::parse();
    let redis_client = redis::Client::open(redis_config.redis_url.clone())
        .expect("Redis 客户端创建失败");
    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Redis 连接失败");
    info!("Redis 连接成功: {}", redis_config.redis_url);

    // jwt
    let jwt_config = lx_dayly_service::config::env::JwtConfig::parse();
    let jwt_secret = jwt_config.jwt_secret;

    let server_config = ServerConfig::parse();
    let port = server_config.get_port();
    let app_env = server_config.app_env();

    // 根据环境配置日志
    if app_env.is_development() {
        // 开发环境：详细日志，美化输出
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .pretty()
            .init();
    } else {
        // 生产环境：精简日志，紧凑格式
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .with_ansi(false)
            .compact()
            .init();
    }

    info!("========================================");
    info!("Environment: {:?}", app_env);
    info!("Server listening on {}:{}", server_config.host, port);
    info!("========================================");

    // 词典 SQLite 数据库
    let dict_dir = server_config.dict_db_dir();
    lx_dayly_service::controller::dict::set_static_dir(&server_config.static_dir());
    lx_dayly_service::controller::dict::init_dict_db(&dict_dir).await;

    // 聊天广播通道
    let (chat_tx, _rx) = tokio::sync::broadcast::channel::<String>(256);

    // WebRTC 信令状态
    let signaling_state = lx_dayly_service::controller::webrtc::SignalingState::new();

    let app = lx_dayly_service::create_app(
        pg_pool,
        jwt_secret,
        server_config.clone(),
        redis_conn,
        chat_tx,
        signaling_state,
    );

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", server_config.host, port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
