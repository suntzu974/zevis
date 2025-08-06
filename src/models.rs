use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsMessage {
    pub id: String,
    pub user: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserNotification {
    pub id: String,
    pub event_type: String,
    pub user_data: User,
    pub timestamp: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CacheValue {
    pub value: String,
    pub ttl: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub name: Option<String>,
}

impl UserNotification {
    pub fn new_created(user: User) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type: "user_created".to_string(),
            message: format!("Nouvel utilisateur créé: {} ({})", user.name, user.email),
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_data: user,
        }
    }

    pub fn new_deleted(user: User) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type: "user_deleted".to_string(),
            message: format!("Utilisateur supprimé: {} ({})", user.name, user.email),
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_data: user,
        }
    }
}
