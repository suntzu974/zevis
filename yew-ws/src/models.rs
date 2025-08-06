use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserData {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserNotification {
    pub event_type: String,
    pub message: String,
    pub user_data: UserData,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WsMessage {
    pub user: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationMessage {
    UserNotification(UserNotification),
    WsMessage(WsMessage),
    Connected,
    Disconnected,
    Error(String),
}

impl NotificationMessage {
    pub fn get_timestamp(&self) -> String {
        match self {
            NotificationMessage::UserNotification(notif) => notif.timestamp.clone(),
            NotificationMessage::WsMessage(msg) => msg.timestamp.clone(),
            _ => chrono::Utc::now().to_rfc3339(),
        }
    }
}
