use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;
use garde::Validate as GardeValidate;

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

#[derive(Debug, Deserialize, Validate, GardeValidate)]
pub struct CreateUserRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    #[garde(length(min = 2, max = 100))]
    pub name: String,
    
    #[validate(email(message = "Invalid email format"))]
    #[garde(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate, GardeValidate)]
pub struct RegistrationRequest {
    #[validate(length(min = 2, max = 100))]
    #[garde(length(min = 2, max = 100))]
    pub name: String,

    #[validate(email)]
    #[garde(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    #[garde(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, GardeValidate)]
pub struct LoginRequest {
    #[validate(email)]
    #[garde(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    #[garde(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Validate, GardeValidate)]
pub struct WsMessage {
    #[validate(length(min = 1, message = "ID cannot be empty"))]
    #[garde(length(min = 1))]
    pub id: String,
    
    #[validate(length(min = 1, max = 50, message = "User name must be between 1 and 50 characters"))]
    #[garde(length(min = 1, max = 50))]
    pub user: String,
    
    #[validate(length(min = 1, max = 1000, message = "Message must be between 1 and 1000 characters"))]
    #[garde(length(min = 1, max = 1000))]
    pub message: String,
    
    #[garde(skip)]
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

#[derive(Debug, Deserialize, Validate, GardeValidate)]
pub struct CacheValue {
    #[validate(length(min = 1, message = "Value cannot be empty"))]
    #[garde(length(min = 1))]
    pub value: String,
    
    #[validate(range(min = 1, max = 86400, message = "TTL must be between 1 second and 24 hours"))]
    #[garde(range(min = 1, max = 86400))]
    pub ttl: Option<u64>,
}

#[derive(Debug, Deserialize, Validate, GardeValidate)]
pub struct QueryParams {
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    #[garde(length(min = 1, max = 100))]
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
