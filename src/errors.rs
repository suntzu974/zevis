use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Email already exists")]
    EmailConflict,
    
    #[error("Cache key not found")]
    CacheKeyNotFound,
    
    #[error("Internal server error")]
    Internal,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AppError::EmailConflict => (StatusCode::CONFLICT, "Email already exists"),
            AppError::CacheKeyNotFound => (StatusCode::NOT_FOUND, "Cache key not found"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad request"),
            AppError::Database(_) | AppError::Redis(_) | AppError::Internal => {
                eprintln!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Serialization(_) => {
                eprintln!("Serialization error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
