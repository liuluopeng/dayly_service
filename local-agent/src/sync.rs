//! gRPC 剪贴板同步客户端 — 将剪贴板事件推送到后端

use std::time::Duration;

use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::Status;
use tracing::info;

pub mod clipboard_sync {
    tonic::include_proto!("clipboard_sync");
}

use clipboard_sync::clipboard_sync_client::ClipboardSyncClient;
use clipboard_sync::{PushClipboardRequest, PushClipboardResponse};

#[derive(Clone)]
pub struct SyncClient {
    client: ClipboardSyncClient<InterceptedService<Channel, AuthInterceptor>>,
}

/// 简单的 token 认证拦截器
#[derive(Clone)]
struct AuthInterceptor;

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        if let Ok(token) = std::env::var("LOCAL_AGENT_TOKEN") {
            req.metadata_mut()
                .insert("authorization", format!("Bearer {}", token).parse().unwrap());
        }
        Ok(req)
    }
}

impl SyncClient {
    /// 连接到后端 gRPC 服务
    pub async fn connect(grpc_addr: &str) -> Result<Self, String> {
        let channel = Channel::from_shared(grpc_addr.to_string())
            .map_err(|e| format!("无效地址: {}", e))?
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await
            .map_err(|e| format!("连接失败: {}", e))?;

        let client = ClipboardSyncClient::with_interceptor(channel, AuthInterceptor);
        info!("gRPC 同步客户端已连接到 {}", grpc_addr);
        Ok(Self { client })
    }

    /// 推送文本到后端
    pub async fn push_text(
        &mut self,
        text: &str,
        hash: &str,
        occurred_at: &str,
    ) -> Result<bool, String> {
        let request = tonic::Request::new(PushClipboardRequest {
            content_type: "text".to_string(),
            text_content: text.to_string(),
            image_path: String::new(),
            image_data: vec![],
            content_hash: hash.to_string(),
            occurred_at: occurred_at.to_string(),
        });

        let response = self
            .client
            .push_clipboard(request)
            .await
            .map_err(|e| format!("gRPC push 失败: {}", e))?
            .into_inner();

        Ok(response.deduplicated)
    }

    /// 推送图片到后端
    pub async fn push_image(
        &mut self,
        image_path: &str,
        image_data: Vec<u8>,
        hash: &str,
        occurred_at: &str,
    ) -> Result<bool, String> {
        let request = tonic::Request::new(PushClipboardRequest {
            content_type: "image".to_string(),
            text_content: String::new(),
            image_path: image_path.to_string(),
            image_data,
            content_hash: hash.to_string(),
            occurred_at: occurred_at.to_string(),
        });

        let response = self
            .client
            .push_clipboard(request)
            .await
            .map_err(|e| format!("gRPC push 失败: {}", e))?
            .into_inner();

        Ok(response.deduplicated)
    }
}
