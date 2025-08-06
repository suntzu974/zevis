use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::Json;
use axum::response::Html;
use serde_json::json;
use tokio::sync::broadcast;

use crate::models::{CreateUserRequest, CacheValue, QueryParams};
use crate::services::{UserService, CacheService};
use crate::errors::Result;

// Application State (Dependency Injection Container)
#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<dyn UserService>,
    pub cache_service: Arc<dyn CacheService>,
    pub broadcast_tx: broadcast::Sender<String>, // Add WebSocket broadcaster
}

// Health Check Handler
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Hello World Handler
pub async fn hello_world(Query(params): Query<QueryParams>) -> &'static str {
    match params.name {
        Some(name) => {
            println!("Hello, {}!", name);
        }
        None => {
            println!("Hello, world!");
        }
    }
    "Hello, world!"
}

// User Handlers
pub async fn get_users(State(state): State<AppState>) -> Result<Json<Vec<crate::models::User>>> {
    let users = state.user_service.get_all_users().await?;
    Ok(Json(users))
}

pub async fn get_user(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<crate::models::User>> {
    let user = state.user_service.get_user_by_id(id).await?;
    Ok(Json(user))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<crate::models::User>> {
    let user = state.user_service.create_user(payload).await?;
    Ok(Json(user))
}

pub async fn delete_user(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<&'static str> {
    state.user_service.delete_user(id).await?;
    Ok("User deleted successfully")
}

// Cache Handlers
pub async fn get_cache(
    Path(key): Path<String>,
    State(state): State<AppState>,
) -> Result<String> {
    state.cache_service.get_cache_value(&key).await
}

pub async fn set_cache(
    Path(key): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<CacheValue>,
) -> Result<&'static str> {
    state.cache_service.set_cache_value(&key, payload).await?;
    Ok("Cache value set successfully")
}

pub async fn delete_cache(
    Path(key): Path<String>,
    State(state): State<AppState>,
) -> Result<&'static str> {
    state.cache_service.delete_cache_value(&key).await?;
    Ok("Cache value deleted successfully")
}

// Yew SPA Handler - serves index.html for all routes with corrected asset paths
pub async fn serve_yew_spa() -> Html<String> {
    match tokio::fs::read_to_string("yew-ws/dist/index.html").await {
        Ok(content) => {
            // Replace absolute paths with /yew-assets/ paths
            let corrected_content = content
                .replace("href=\"/yew-ws-notifications-", "href=\"/yew-assets/yew-ws-notifications-")
                .replace("src=\"/yew-ws-notifications-", "src=\"/yew-assets/yew-ws-notifications-");
            Html(corrected_content)
        },
        Err(_) => Html("<html><body><h1>Yew app not found</h1><p>Please build the Yew app first with <code>trunk build --release</code></p></body></html>".to_string()),
    }
}
