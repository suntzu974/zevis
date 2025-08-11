use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const JWT_SECRET: &[u8] = b"your-secret-key"; // In production, use env variable

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user id)
    pub exp: i64,    // Expiration time
    pub iat: i64,    // Issued at
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterPayload {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, Json(self)).into_response()
    }
}

impl Claims {
    pub fn new(user_id: String, email: String, role: String) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(24)).timestamp();
        let iat = now.timestamp();

        Self {
            sub: user_id,
            exp,
            iat,
            email,
            role,
        }
    }

    pub fn encode(&self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(JWT_SECRET),
        )
    }

    pub fn decode(token: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let validation = Validation::default();
        
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET),
            &validation,
        )
        .map(|data| data.claims)
    }
}

// Password hashing functions
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

// Helper function to extract claims from request headers manually
pub fn extract_claims_from_auth_header(auth_header: &str) -> Result<Claims, String> {
    if !auth_header.starts_with("Bearer ") {
        return Err("Invalid authorization header format".to_string());
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix
    Claims::decode(token).map_err(|_| "Invalid or expired token".to_string())
}
