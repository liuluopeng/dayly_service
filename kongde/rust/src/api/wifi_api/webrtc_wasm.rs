// WASM stub: webrtc-rs 依赖 ring 不支持 wasm32，Web 端用浏览器原生 WebRTC
use crate::frb_generated::StreamSink;

#[allow(unused_variables)]
pub fn connect_webrtc(sink: StreamSink<String>, device_name: String) {
    // WebRTC 在浏览器端由原生 API 处理
}

#[allow(unused_variables)]
pub async fn send_webrtc_message(msg: String) -> Result<(), String> {
    Err("WebRTC 在 Web 端不可用".to_string())
}
