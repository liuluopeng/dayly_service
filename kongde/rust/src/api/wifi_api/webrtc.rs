use crate::frb_generated::StreamSink;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use serde::{Deserialize, Serialize};

use super::init::get_client_clone;

/// 共享 Tokio runtime（FRB 不在 Tokio runtime 中）
fn shared_rt() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().expect("创建 Tokio runtime 失败"))
}

/// 信令消息（从服务器接收）
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum SignalResponse {
    #[serde(rename = "registered")]
    Registered { peer_id: String },
    #[serde(rename = "answer")]
    Answer { sdp: String },
    #[serde(rename = "candidate")]
    Candidate {
        candidate: String,
        sdp_mid: String,
        sdp_mline_index: u16,
    },
    #[serde(rename = "peer_joined")]
    PeerJoined { id: String, name: String },
    #[serde(rename = "peer_left")]
    PeerLeft { id: String },
    #[serde(rename = "peer_list")]
    PeerList { peers: Vec<PeerListItem> },
}

#[derive(Debug, Deserialize)]
struct PeerListItem {
    id: String,
    name: String,
}

/// 信令消息（发送给服务器）
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum SignalRequest {
    #[serde(rename = "register")]
    Register { name: String },
    #[serde(rename = "offer")]
    Offer { sdp: String },
    #[serde(rename = "candidate")]
    Candidate {
        candidate: String,
        sdp_mid: String,
        sdp_mline_index: u16,
    },
}

/// 全局 DataChannel 引用，用于发送消息
static GLOBAL_DC: once_cell::sync::OnceCell<Arc<Mutex<Option<Arc<RTCDataChannel>>>>> =
    once_cell::sync::OnceCell::new();

fn get_dc_holder() -> &'static Arc<Mutex<Option<Arc<RTCDataChannel>>>> {
    GLOBAL_DC.get_or_init(|| Arc::new(Mutex::new(None)))
}

/// 连接 WebRTC 信令并建立 DataChannel。
/// 消息通过 StreamSink 推送给 Dart。
pub fn connect_webrtc(sink: StreamSink<String>, device_name: String) {
    let client = match get_client_clone() {
        Ok(c) => c,
        Err(e) => {
            log::error!("WebRTC: 获取客户端失败: {}", e);
            return;
        }
    };
    let base_url = client.base_url().to_string();
    let token = client.token().unwrap_or_default().to_string();

    let ws_url = base_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let full_url = format!(
        "{}/api/webrtc/signaling?token={}",
        ws_url,
        urlencoding::encode(&token)
    );

    let handle = shared_rt().handle().clone();
    std::thread::spawn(move || {
        let _guard = handle.enter();
        handle.block_on(async move {
        let request = match full_url.into_client_request() {
            Ok(r) => r,
            Err(e) => {
                log::error!("WebRTC: 无效 URL: {}", e);
                return;
            }
        };

        let ws_stream = match connect_async(request).await {
            Ok((ws, _)) => ws,
            Err(e) => {
                log::error!("WebRTC: 连接信令失败: {}", e);
                return;
            }
        };

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // 发送 register 消息
        let register = serde_json::to_string(&SignalRequest::Register {
            name: device_name,
        })
        .unwrap();
        if ws_sender
            .send(tokio_tungstenite::tungstenite::Message::Text(register.into()))
            .await
            .is_err()
        {
            log::error!("WebRTC: 发送 register 失败");
            return;
        }

        // 等待 registered 确认
        let registered_msg = match ws_receiver.next().await {
            Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => text.to_string(),
            _ => {
                log::error!("WebRTC: 未收到 registered 确认");
                return;
            }
        };

        let resp: SignalResponse = match serde_json::from_str(&registered_msg) {
            Ok(SignalResponse::Registered { peer_id }) => {
                log::info!("WebRTC: 注册成功, peer_id={}", peer_id);
                SignalResponse::Registered { peer_id }
            }
            _ => {
                log::error!("WebRTC: 收到非 registered 消息");
                return;
            }
        };

        // 创建 PeerConnection
        let api = APIBuilder::new().build();
        let peer_connection = match api.new_peer_connection(RTCConfiguration::default()).await {
            Ok(pc) => Arc::new(pc),
            Err(e) => {
                log::error!("WebRTC: 创建 PeerConnection 失败: {}", e);
                return;
            }
        };

        // 创建 DataChannel
        let dc = match peer_connection
            .create_data_channel("sharing", None)
            .await
        {
            Ok(dc) => dc,
            Err(e) => {
                log::error!("WebRTC: 创建 DataChannel 失败: {}", e);
                return;
            }
        };

        // DataChannel on_message — 推送给 Dart
        let sink_msg = sink.clone();
        dc.on_message(Box::new(move |msg: DataChannelMessage| {
            let text = String::from_utf8_lossy(&msg.data).to_string();
            let _ = sink_msg.add(text);
            Box::pin(async {})
        }));

        // DataChannel on_open — 保存全局引用，通知 Dart
        let dc_holder = get_dc_holder().clone();
        let dc_for_open = dc.clone();
        let sink_open = sink.clone();
        dc.on_open(Box::new(move || {
            let dc_holder = dc_holder.clone();
            let dc = dc_for_open.clone();
            let sink = sink_open.clone();
            Box::pin(async move {
                log::info!("WebRTC DataChannel 已打开");
                let mut holder = dc_holder.lock().await;
                *holder = Some(dc);
                let _ = sink.add(r#"{"type":"connected"}"#.to_string());
            })
        }));

        // ICE candidate 处理器
        let ws_sender_ice = Arc::new(Mutex::new(ws_sender));
        let ws_sender_for_ice = ws_sender_ice.clone();

        peer_connection.on_ice_candidate(Box::new(move |candidate| {
            let ws_sender = ws_sender_for_ice.clone();
            Box::pin(async move {
                if let Some(c) = candidate {
                    if let Ok(init) = c.to_json() {
                        let req = SignalRequest::Candidate {
                            candidate: init.candidate,
                            sdp_mid: init.sdp_mid.unwrap_or_default(),
                            sdp_mline_index: init.sdp_mline_index.unwrap_or(0),
                        };
                        let mut sender = ws_sender.lock().await;
                        let _ = sender
                            .send(tokio_tungstenite::tungstenite::Message::Text(
                                serde_json::to_string(&req).unwrap().into(),
                            ))
                            .await;
                    }
                }
            })
        }));

        // 创建 Offer
        let offer = match peer_connection.create_offer(None).await {
            Ok(o) => o,
            Err(e) => {
                log::error!("WebRTC: 创建 offer 失败: {}", e);
                return;
            }
        };

        if let Err(e) = peer_connection.set_local_description(offer.clone()).await {
            log::error!("WebRTC: 设置 local description 失败: {}", e);
            return;
        }

        // 发送 Offer
        let offer_req = SignalRequest::Offer {
            sdp: offer.sdp.to_string(),
        };
        {
            let mut sender = ws_sender_ice.lock().await;
            let _ = sender
                .send(tokio_tungstenite::tungstenite::Message::Text(
                    serde_json::to_string(&offer_req).unwrap().into(),
                ))
                .await;
        }

        // 处理后续信令消息（answer, candidate）
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                tokio_tungstenite::tungstenite::Message::Text(text) => {
                    let sig: SignalResponse = match serde_json::from_str(&text) {
                        Ok(s) => s,
                        Err(e) => {
                            log::error!("WebRTC: 解析信令失败: {}", e);
                            continue;
                        }
                    };

                    match sig {
                        SignalResponse::Answer { sdp } => {
                            log::info!("WebRTC: 收到 Answer");
                            let answer = RTCSessionDescription::answer(sdp).unwrap();
                            if let Err(e) =
                                peer_connection.set_remote_description(answer).await
                            {
                                log::error!("WebRTC: 设置 remote description 失败: {}", e);
                            }
                        }
                        SignalResponse::Candidate {
                            candidate,
                            sdp_mid,
                            sdp_mline_index,
                        } => {
                            let ice = RTCIceCandidateInit {
                                candidate,
                                sdp_mid: Some(sdp_mid),
                                sdp_mline_index: Some(sdp_mline_index),
                                username_fragment: None,
                            };
                            if let Err(e) = peer_connection.add_ice_candidate(ice).await {
                                log::error!("WebRTC: 添加 ICE candidate 失败: {}", e);
                            }
                        }
                        SignalResponse::PeerJoined { id, name } => {
                            log::info!("WebRTC: 设备 '{}' ({}) 加入", name, id);
                            let notification = serde_json::json!({
                                "type": "peer_joined",
                                "id": id,
                                "name": name
                            });
                            let _ = sink.add(notification.to_string());
                        }
                        SignalResponse::PeerLeft { id } => {
                            log::info!("WebRTC: 设备 {} 离开", id);
                            let notification = serde_json::json!({
                                "type": "peer_left",
                                "id": id
                            });
                            let _ = sink.add(notification.to_string());
                        }
                        SignalResponse::PeerList { peers } => {
                            log::info!("WebRTC: 收到在线列表, {} 个设备", peers.len());
                            let notification = serde_json::json!({
                                "type": "peer_list",
                                "peers": peers.iter().map(|p| {
                                    serde_json::json!({"id": p.id, "name": p.name})
                                }).collect::<Vec<_>>()
                            });
                            let _ = sink.add(notification.to_string());
                        }
                        _ => {}
                    }
                }
                tokio_tungstenite::tungstenite::Message::Close(_) => break,
                _ => {}
            }
        }

        // 清理
        log::info!("WebRTC: 信令连接关闭");
        let _ = peer_connection.close().await;
        let mut holder = get_dc_holder().lock().await;
        *holder = None;
        });
    });
}

/// 通过 DataChannel 发送消息（JSON 字符串）
pub async fn send_webrtc_message(msg: String) -> Result<(), String> {
    shared_rt().block_on(async {
        let holder = get_dc_holder().lock().await;
        if let Some(dc) = holder.as_ref() {
            dc.send_text(msg)
                .await
                .map(|_| ())
                .map_err(|e| format!("发送失败: {}", e))
        } else {
            Err("DataChannel 未连接".to_string())
        }
    })
}
