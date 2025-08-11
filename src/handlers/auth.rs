use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json},
};
use serde_json::json;

use crate::{
    auth::{AuthPayload, AuthResponse, Claims, RegisterPayload, UserInfo, hash_password, verify_password, extract_claims_from_auth_header},
    handlers::AppState,
    models::User,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Check if user already exists
    if let Ok(_) = state.user_service.get_user_by_email(&payload.email).await {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "User already exists"})),
        ));
    }

    // Hash password
    let password_hash = hash_password(&payload.password)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to hash password"})),
            )
        })?;

    // Create user
    let new_user = User {
        id: 0, // Will be set by database
        name: payload.name,
        email: payload.email.clone(),
        password_hash: Some(password_hash),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created_user = state
        .user_service
        .create_user_with_password(new_user)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create user"})),
            )
        })?;

    // Generate JWT token
    let claims = Claims::new(
        created_user.id.to_string(),
        created_user.email.clone(),
        "user".to_string(),
    );

    let token = claims.encode().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to generate token"})),
        )
    })?;

    let response = AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        user: UserInfo {
            id: created_user.id.to_string(),
            name: created_user.name.clone(),
            email: created_user.email.clone(),
            role: "user".to_string(),
        },
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Get user by email
    let user = state
        .user_service
        .get_user_by_email(&payload.email)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid credentials"})),
            )
        })?;

    // Verify password
    let password_hash = user.password_hash.as_ref().ok_or((
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": "Invalid credentials"})),
    ))?;

    let is_valid = verify_password(&payload.password, password_hash)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to verify password"})),
            )
        })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"})),
        ));
    }

    // Generate JWT token
    let claims = Claims::new(
        user.id.to_string(),
        user.email.clone(),
        "user".to_string(),
    );

    let token = claims.encode().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to generate token"})),
        )
    })?;

    let response = AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        user: UserInfo {
            id: user.id.to_string(),
            name: user.name.clone(),
            email: user.email.clone(),
            role: "user".to_string(),
        },
    };

    Ok(Json(response))
}

pub async fn me(headers: HeaderMap) -> impl IntoResponse {
    let auth_header = match headers.get("authorization").and_then(|h| h.to_str().ok()) {
        Some(header) => header,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing authorization header"})),
            ).into_response();
        }
    };

    let claims = match extract_claims_from_auth_header(auth_header) {
        Ok(claims) => claims,
        Err(err) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": err})),
            ).into_response();
        }
    };

    Json(json!({
        "id": claims.sub,
        "email": claims.email,
        "role": claims.role
    })).into_response()
}

pub async fn protected(headers: HeaderMap) -> impl IntoResponse {
    let auth_header = match headers.get("authorization").and_then(|h| h.to_str().ok()) {
        Some(header) => header,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing authorization header"})),
            ).into_response();
        }
    };

    let claims = match extract_claims_from_auth_header(auth_header) {
        Ok(claims) => claims,
        Err(err) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": err})),
            ).into_response();
        }
    };

    Json(json!({
        "message": "This is a protected endpoint",
        "user": {
            "id": claims.sub,
            "email": claims.email,
            "role": claims.role
        }
    })).into_response()
}
