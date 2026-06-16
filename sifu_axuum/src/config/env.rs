use std::net::IpAddr;

use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

impl AppEnv {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => AppEnv::Production,
            _ => AppEnv::Development,
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            AppEnv::Development => 23001,
            AppEnv::Production => 23000,
        }
    }

    pub fn is_development(&self) -> bool {
        matches!(self, AppEnv::Development)
    }
}

#[derive(Debug, Parser, Clone)]
pub struct ServerConfig {
    #[clap(default_value = "127.0.0.1", env)]
    pub host: IpAddr,
    #[clap(default_value = "23000", env)]
    pub port: u16,
    #[clap(default_value = "production", env)]
    pub env: String,
    #[clap(default_value = "192.168.31.58", env)]
    pub domain: String,
}

impl ServerConfig {
    pub fn app_env(&self) -> AppEnv {
        AppEnv::from_str(&self.env)
    }

    pub fn get_port(&self) -> u16 {
        if self.port == 23000 && self.app_env() == AppEnv::Development {
            self.app_env().default_port()
        } else {
            self.port
        }
    }

    pub fn get_base_url(&self) -> String {
        format!("http://{}:{}", self.domain, self.get_port())
    }

    pub fn static_dir(&self) -> String {
        match self.app_env() {
            AppEnv::Development => format!("{}/static", env!("CARGO_MANIFEST_DIR")),
            AppEnv::Production => "/app/static".to_string(),
        }
    }

    pub fn dict_db_dir(&self) -> String {
        std::env::var("DICT_DB_DIR").unwrap_or_else(|_| {
            if self.app_env().is_development() { "cold_data".into() }
            else { "/app/cold_data".into() }
        })
    }
}

#[derive(Debug, Parser)]
pub struct PgConfig {
    #[clap(required = true, env)]
    pub pg_database: String,
    #[clap(default_value = "localhost", env)]
    pub pg_host: String,
    #[clap(default_value = "5432", env)]
    pub pg_port: u16,
    #[clap(default_value = "postgres", env)]
    pub pg_user: String,
    #[clap(default_value = "", env)]
    pub pg_password: String,
}

#[derive(Debug, Parser)]
pub struct JwtConfig {
    #[clap(default_value = "jwt_secret", env)]
    pub jwt_secret: String,
}

#[derive(Debug, Parser)]
pub struct StreamConfig {
    #[clap(default_value = "/tmp/stream", env)]
    pub stream_root_dir: String,
}

#[derive(Debug, Parser, Clone)]
pub struct OpenAiConfig {
    #[clap(default_value = "https://api.deepseek.com", env)]
    pub openai_base_url: String,
    #[clap(default_value = "", env)]
    pub openai_api_key: String,
}

#[derive(Debug, Parser, Clone)]
pub struct RedisConfig {
    #[clap(default_value = "redis://127.0.0.1:6379", env)]
    pub redis_url: String,
}
