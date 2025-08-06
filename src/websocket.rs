use axum::extract::ws::{WebSocket, Message};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use uuid::Uuid;
use serde_json;

use crate::models::WsMessage;
use crate::errors::Result;
use crate::handlers::AppState; // Use unified state

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

pub async fn websocket_connection(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.broadcast_tx.subscribe();
    
    let broadcast_tx = state.broadcast_tx.clone();
    
    // Handle incoming messages
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                if let Err(e) = handle_websocket_message(msg, &broadcast_tx).await {
                    eprintln!("WebSocket message handling error: {}", e);
                }
            } else {
                break;
            }
        }
    });
    
    // Handle outgoing messages
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }
}

async fn handle_websocket_message(
    msg: Message,
    broadcast_tx: &broadcast::Sender<String>,
) -> Result<()> {
    match msg {
        Message::Text(text) => {
            println!("Received WebSocket message: {}", text);
            
            let ws_message = if let Ok(parsed_msg) = serde_json::from_str::<WsMessage>(&text) {
                parsed_msg
            } else {
                // Create a simple message if parsing fails
                WsMessage {
                    id: Uuid::new_v4().to_string(),
                    user: "anonymous".to_string(),
                    message: text,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }
            };
            
            // Broadcast to all connected clients
            if let Ok(msg_json) = serde_json::to_string(&ws_message) {
                let _ = broadcast_tx.send(msg_json);
            }
        }
        Message::Binary(_) => {
            println!("Received binary WebSocket message");
        }
        Message::Close(_) => {
            println!("WebSocket connection closed");
        }
        _ => {}
    }
    
    Ok(())
}
