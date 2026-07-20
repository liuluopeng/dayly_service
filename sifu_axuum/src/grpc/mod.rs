//! gRPC 服务实现 — 剪贴板历史
//!
//! 由 proto/clipboard.proto 编译生成桩代码，在此实现业务逻辑。

pub mod clipboard {
    tonic::include_proto!("clipboard");
}

pub static CLIPBOARD_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("clipboard_descriptor");

pub static CLIPBOARD_SYNC_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("clipboard_sync_descriptor");

use clipboard::clipboard_history_server::{ClipboardHistory, ClipboardHistoryServer};
use clipboard::{HistoryRequest, HistoryResponse};

// ─── ClipboardHistory 服务 ─────────────────────────────────────

pub struct ClipboardHistorySvc;

#[tonic::async_trait]
impl ClipboardHistory for ClipboardHistorySvc {
    async fn get_history(
        &self,
        _request: tonic::Request<HistoryRequest>,
    ) -> Result<tonic::Response<HistoryResponse>, tonic::Status> {
        Ok(tonic::Response::new(HistoryResponse {
            code: 200,
            message: "ok".into(),
            data: vec![],
        }))
    }
}

/// 构建 Tonic gRPC 服务，准备嵌入 axum
pub fn clipboard_grpc_service() -> ClipboardHistoryServer<ClipboardHistorySvc> {
    ClipboardHistoryServer::new(ClipboardHistorySvc)
}

// ─── Hello 服务 ────────────────────────────────────────────────

pub mod hello {
    tonic::include_proto!("hello");
}

pub static HELLO_DESCRIPTOR: &[u8] =
    tonic::include_file_descriptor_set!("hello_descriptor");

use hello::greeter_server::{Greeter, GreeterServer};
use hello::{HelloRequest, HelloResponse};

#[derive(Debug, Default)]
pub struct GreeterSvc;

#[tonic::async_trait]
impl Greeter for GreeterSvc {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloResponse>, tonic::Status> {
        let name = request.into_inner().name;
        let message = if name.is_empty() {
            "Hello, World!".to_string()
        } else {
            format!("Hello, {}!", name)
        };
        Ok(tonic::Response::new(HelloResponse { message }))
    }
}

pub fn hello_grpc_service() -> GreeterServer<GreeterSvc> {
    GreeterServer::new(GreeterSvc)
}

// ─── ClipboardSync 服务 ────────────────────────────────────────
// local-agent 通过此接口推送剪贴板事件

pub mod clipboard_sync {
    tonic::include_proto!("clipboard_sync");
}

use clipboard_sync::clipboard_sync_server::{ClipboardSync, ClipboardSyncServer};
use clipboard_sync::{PushClipboardRequest, PushClipboardResponse};

#[derive(Debug, Default)]
pub struct ClipboardSyncSvc;

#[tonic::async_trait]
impl ClipboardSync for ClipboardSyncSvc {
    async fn push_clipboard(
        &self,
        request: tonic::Request<PushClipboardRequest>,
    ) -> Result<tonic::Response<PushClipboardResponse>, tonic::Status> {
        let req = request.into_inner();
        tracing::info!("📋 gRPC 收到剪贴板: {} ({})", req.content_type, req.text_content.chars().take(40).collect::<String>());
        Ok(tonic::Response::new(PushClipboardResponse {
            code: 200,
            message: "ok".into(),
            deduplicated: false,
        }))
    }
}

pub fn clipboard_sync_grpc_service() -> ClipboardSyncServer<ClipboardSyncSvc> {
    ClipboardSyncServer::new(ClipboardSyncSvc)
}
