use crate::frb_generated::StreamSink;
use futures::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use super::init::get_client_clone;

/// Connect to the chat WebSocket and stream messages to Dart.
/// The stream ends when the Dart side cancels the subscription.
pub fn connect_chat_ws(sink: StreamSink<String>, path: String) {
    let client = match get_client_clone() {
        Ok(c) => c,
        Err(e) => {
            log::error!("WS: failed to get client: {}", e);
            return;
        }
    };
    let base_url = client.base_url().to_string();
    let token = client.token().unwrap_or_default().to_string();

    let ws_url = base_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let full_url = format!("{}{}?token={}", ws_url, path, urlencoding::encode(&token));

    tokio::spawn(async move {
        let request = match full_url.into_client_request() {
            Ok(r) => r,
            Err(e) => {
                log::error!("WS invalid URL: {}", e);
                return;
            }
        };

        loop {
            let ws_stream = match connect_async(request.clone()).await {
                Ok((ws, _)) => ws,
                Err(e) => {
                    log::error!("WS connect failed: {}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    continue;
                }
            };

            let (_, mut read) = ws_stream.split();

            loop {
                match read.next().await {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                        if sink.add(text.to_string()).is_err() {
                            // Dart side dropped the stream
                            return;
                        }
                    }
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => break,
                    Some(Err(_)) => break,
                    None => break,
                    _ => {}
                }
            }

            // Connection lost, reconnect after delay
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });
}
