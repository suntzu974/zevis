use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;
use garde::Report as GardeReport;

// Problem Details RFC 7807 compliant error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: Option<String>,
    pub instance: Option<String>,
    #[serde(flatten)]
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

impl ProblemDetails {
    pub fn new(problem_type: &str, title: &str, status: u16) -> Self {
        Self {
            problem_type: problem_type.to_string(),
            title: title.to_string(),
            status,
            detail: None,
            instance: None,
            extensions: None,
        }
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    pub fn with_instance(mut self, instance: &str) -> Self {
        self.instance = Some(instance.to_string());
        self
    }

    pub fn with_extension(mut self, key: &str, value: serde_json::Value) -> Self {
        let mut extensions = self.extensions.unwrap_or_default();
        extensions.insert(key.to_string(), value);
        self.extensions = Some(extensions);
        self
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Validation error")]
    ValidationError(ValidationErrors),
    
    #[error("Garde validation error")]
    GardeValidation(GardeReport),
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Email already exists")]
    EmailConflict,
    
    #[error("Cache key not found")]
    CacheKeyNotFound,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let problem_details = match self {
            AppError::ValidationError(errors) => {
                let mut extensions = serde_json::Map::new();
                extensions.insert("validation_errors".to_string(), json!(errors));
                
                ProblemDetails::new(
                    "https://example.com/probs/validation-error",
                    "Validation Failed",
                    400
                ).with_detail("One or more validation errors occurred.")
                .with_extension("validation_errors", json!(errors))
            },
            AppError::GardeValidation(report) => {
                ProblemDetails::new(
                    "https://example.com/probs/validation-error", 
                    "Validation Failed",
                    400
                ).with_detail(&format!("Garde validation failed: {}", report))
            },
            AppError::UserNotFound => {
                ProblemDetails::new(
                    "https://example.com/probs/not-found",
                    "User Not Found", 
                    404
                ).with_detail("The requested user could not be found.")
            },
            AppError::EmailConflict => {
                ProblemDetails::new(
                    "https://example.com/probs/conflict",
                    "Email Conflict",
                    409
                ).with_detail("An account with this email address already exists.")
            },
            AppError::CacheKeyNotFound => {
                ProblemDetails::new(
                    "https://example.com/probs/not-found",
                    "Cache Key Not Found",
                    404
                ).with_detail("The requested cache key could not be found.")
            },
            AppError::BadRequest(msg) => {
                ProblemDetails::new(
                    "https://example.com/probs/bad-request",
                    "Bad Request",
                    400
                ).with_detail(&msg)
            },
            AppError::Unauthorized(msg) => {
                ProblemDetails::new(
                    "https://example.com/probs/unauthorized",
                    "Unauthorized",
                    401
                ).with_detail(&msg)
            },
            AppError::Forbidden(msg) => {
                ProblemDetails::new(
                    "https://example.com/probs/forbidden",
                    "Forbidden",
                    403
                ).with_detail(&msg)
            },
            AppError::Database(_) | AppError::Redis(_) | AppError::Serialization(_) | AppError::Internal => {
                eprintln!("Internal error: {}", self);
                ProblemDetails::new(
                    "https://example.com/probs/internal-error",
                    "Internal Server Error",
                    500
                ).with_detail("An unexpected error occurred. Please try again later.")
            },
        };

        problem_details.into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
