use axum::{
    Extension, Json, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::get,
};
use common::api::base::{ApiError, ApiResponse, ApiResult};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use uuid::Uuid;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

type WsSender = Arc<Mutex<futures::stream::SplitSink<WebSocket, Message>>>;

/// WebRTC 信令状态
#[derive(Clone)]
pub struct SignalingState {
    /// peer_id -> DataChannel（用于数据转发）
    peers: Arc<Mutex<HashMap<String, Arc<RTCDataChannel>>>>,
    /// peer_id -> WebSocket sender（用于发送在线列表等信令）
    ws_senders: Arc<Mutex<HashMap<String, WsSender>>>,
    /// peer_id -> device_name
    names: Arc<Mutex<HashMap<String, String>>>,
}

impl SignalingState {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashMap::new())),
            ws_senders: Arc::new(Mutex::new(HashMap::new())),
            names: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn set_name(&self, id: String, name: String) {
        self.names.lock().await.insert(id, name);
    }

    async fn remove_peer(&self, id: &str) {
        self.peers.lock().await.remove(id);
        self.ws_senders.lock().await.remove(id);
        self.names.lock().await.remove(id);
    }

    /// 通过 DataChannel 广播数据给其他 peer
    async fn broadcast_data(&self, sender_id: &str, msg: &str) {
        let peers = self.peers.lock().await;
        for (id, dc) in peers.iter() {
            if id != sender_id {
                let _ = dc.send_text(msg.to_string()).await;
            }
        }
    }

    /// 通过 WebSocket 广播在线列表给所有 peer
    async fn broadcast_peer_list(&self) {
        let names = self.names.lock().await;
        let peer_list: Vec<PeerInfo> = names
            .iter()
            .map(|(id, name)| PeerInfo {
                id: id.clone(),
                name: name.clone(),
            })
            .collect();
        drop(names);

        let msg = serde_json::to_string(&SignalingResponse::PeerList {
            peers: peer_list,
        })
        .unwrap();

        let senders = self.ws_senders.lock().await;
        for (_, sender) in senders.iter() {
            let mut s = sender.lock().await;
            let _ = s.send(Message::Text(msg.clone().into())).await;
        }
    }

    async fn get_peer_list(&self) -> Vec<PeerInfo> {
        let names = self.names.lock().await;
        names
            .iter()
            .map(|(id, name)| PeerInfo {
                id: id.clone(),
                name: name.clone(),
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
struct PeerInfo {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum SignalingMessage {
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

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum SignalingResponse {
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
    PeerList { peers: Vec<PeerInfo> },
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<SignalingState>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: SignalingState) {
    let (ws_sender, mut ws_receiver) = socket.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));
    let peer_id = Uuid::new_v4().to_string();
    let mut device_name = String::new();

    // 等待客户端发送 register 消息
    let register_msg = match ws_receiver.next().await {
        Some(Ok(Message::Text(text))) => text,
        _ => {
            info!("WebRTC signaling: 客户端未发送 register 消息就断开");
            return;
        }
    };

    match serde_json::from_str(&register_msg) {
        Ok(SignalingMessage::Register { name }) => {
            device_name = name;
        }
        _ => {
            info!("WebRTC signaling: 首条消息不是 register");
            return;
        }
    };

    info!("WebRTC signaling: 设备 '{}' 注册, peer_id={}", device_name, peer_id);

    // 保存设备名和 WebSocket sender
    state.set_name(peer_id.clone(), device_name.clone()).await;
    state.ws_senders.lock().await.insert(peer_id.clone(), ws_sender.clone());

    // 发送注册确认
    let resp = SignalingResponse::Registered {
        peer_id: peer_id.clone(),
    };
    {
        let mut sender = ws_sender.lock().await;
        if sender
            .send(Message::Text(serde_json::to_string(&resp).unwrap().into()))
            .await
            .is_err()
        {
            return;
        }
    }

    // 广播更新后的在线列表给所有人（通过 WebSocket）
    state.broadcast_peer_list().await;

    // 创建 WebRTC PeerConnection
    let api = APIBuilder::new().build();
    let peer_connection = match api
        .new_peer_connection(RTCConfiguration::default())
        .await
    {
        Ok(pc) => Arc::new(pc),
        Err(e) => {
            error!("创建 PeerConnection 失败: {}", e);
            return;
        }
    };

    // DataChannel: 只用于数据转发
    let state_dc = state.clone();
    let peer_id_dc = peer_id.clone();
    peer_connection.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
        let state = state_dc.clone();
        let peer_id = peer_id_dc.clone();

        info!("WebRTC on_data_channel 触发, peer={}", peer_id);

        // on_message — 转发给其他 peer 的 DataChannel
        let state_msg = state.clone();
        let peer_id_msg = peer_id.clone();
        dc.on_message(Box::new(move |msg| {
            let text = String::from_utf8_lossy(&msg.data).to_string();
            let state = state_msg.clone();
            let peer_id = peer_id_msg.clone();
            Box::pin(async move {
                state.broadcast_data(&peer_id, &text).await;
            })
        }));

        // on_open — 保存 DataChannel 引用（在线状态通过 WebSocket 管理）
        let state_open = state.clone();
        let peer_id_open = peer_id.clone();
        let dc_open = dc.clone();
        dc.on_open(Box::new(move || {
            let state = state_open.clone();
            let peer_id = peer_id_open.clone();
            let dc = dc_open.clone();
            Box::pin(async move {
                info!("WebRTC DataChannel 打开, peer={}", peer_id);
                state.peers.lock().await.insert(peer_id, dc);
            })
        }));

        // on_close — 移除 DataChannel 引用
        let state_close = state.clone();
        let peer_id_close = peer_id.clone();
        dc.on_close(Box::new(move || {
            let state = state_close.clone();
            let peer_id = peer_id_close.clone();
            Box::pin(async move {
                info!("WebRTC DataChannel 关闭, peer={}", peer_id);
                state.peers.lock().await.remove(&peer_id);
            })
        }));

        Box::pin(async {})
    }));

    // ICE candidate 处理器
    let ws_sender_for_ice = ws_sender.clone();

    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        let ws_sender = ws_sender_for_ice.clone();
        Box::pin(async move {
            if let Some(c) = candidate {
                if let Ok(init) = c.to_json() {
                    let resp = SignalingResponse::Candidate {
                        candidate: init.candidate,
                        sdp_mid: init.sdp_mid.unwrap_or_default(),
                        sdp_mline_index: init.sdp_mline_index.unwrap_or(0),
                    };
                    let mut sender = ws_sender.lock().await;
                    let _ = sender
                        .send(Message::Text(serde_json::to_string(&resp).unwrap().into()))
                        .await;
                }
            }
        })
    }));

    // 处理信令消息（offer, candidate）
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                let sig_msg: SignalingMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("解析信令消息失败: {}", e);
                        continue;
                    }
                };

                match sig_msg {
                    SignalingMessage::Offer { sdp } => {
                        info!("WebRTC: 收到 Offer, peer={}", peer_id);

                        let offer = RTCSessionDescription::offer(sdp).unwrap();
                        if let Err(e) = peer_connection.set_remote_description(offer).await {
                            error!("设置 remote description 失败: {}", e);
                            continue;
                        }

                        let answer = match peer_connection.create_answer(None).await {
                            Ok(a) => a,
                            Err(e) => {
                                error!("创建 answer 失败: {}", e);
                                continue;
                            }
                        };

                        if let Err(e) = peer_connection
                            .set_local_description(answer.clone())
                            .await
                        {
                            error!("设置 local description 失败: {}", e);
                            continue;
                        }

                        let resp = SignalingResponse::Answer {
                            sdp: answer.sdp.to_string(),
                        };
                        let mut sender = ws_sender.lock().await;
                        let _ = sender
                            .send(
                                Message::Text(serde_json::to_string(&resp).unwrap().into()),
                            )
                            .await;
                    }
                    SignalingMessage::Candidate {
                        candidate,
                        sdp_mid,
                        sdp_mline_index,
                    } => {
                        let ice_candidate = RTCIceCandidateInit {
                            candidate,
                            sdp_mid: Some(sdp_mid),
                            sdp_mline_index: Some(sdp_mline_index),
                            username_fragment: None,
                        };
                        if let Err(e) = peer_connection
                            .add_ice_candidate(ice_candidate)
                            .await
                        {
                            error!("添加 ICE candidate 失败: {}", e);
                        }
                    }
                    SignalingMessage::Register { .. } => {
                        // 已处理过
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // 连接断开，清理并广播在线列表
    info!("WebRTC signaling: 设备 '{}' 断开", device_name);
    state.remove_peer(&peer_id).await;
    let _ = peer_connection.close().await;
    state.broadcast_peer_list().await;
}

async fn get_peers(
    Extension(state): Extension<SignalingState>,
) -> ApiResult<ApiResponse<Vec<PeerInfo>>> {
    let peers = state.get_peer_list().await;
    Ok(ApiResponse::ok(peers))
}

pub fn webrtc_routes() -> Router {
    Router::new()
        .route("/signaling", get(ws_handler))
        .route("/peers", get(get_peers))
}
