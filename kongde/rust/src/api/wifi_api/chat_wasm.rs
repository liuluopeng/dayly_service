// WASM stub: 浏览器端 WebSocket 不走 Rust（Vue 用 web-sys，Flutter Web 暂无）
use crate::frb_generated::StreamSink;

#[allow(unused_variables)]
pub fn connect_chat_ws(sink: StreamSink<String>, path: String) {
    // Web 端不需要 chat WebSocket（由浏览器原生处理）
}
