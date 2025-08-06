use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WsMessage {
    pub id: String,
    pub user: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserNotification {
    pub id: String,
    pub event_type: String,
    pub user_data: User,
    pub timestamp: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum NotificationMessage {
    WsMessage(WsMessage),
    UserNotification(UserNotification),
    Connected,
    Disconnected,
    Error(String),
}
