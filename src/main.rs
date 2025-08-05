use axum::{
    extract::{Query, State, WebSocketUpgrade, ws::{WebSocket, Message}},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post, delete},
    Router,
};
use redis::{aio::ConnectionManager, Client};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use uuid::Uuid;
use tokio::sync::broadcast;
use futures_util::{SinkExt, StreamExt};

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
struct User {
    id: i32,
    name: String,
    email: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct Params {
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct WsMessage {
    id: String,
    user: String,
    message: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct UserNotification {
    id: String,
    event_type: String, // "user_created", "user_deleted", etc.
    user_data: User,
    timestamp: String,
    message: String,
}

// Application state containing database pool, Redis connection and broadcast channel
#[derive(Clone)]
struct AppState {
    db: PgPool,
    redis: ConnectionManager,
    broadcast_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/zevis".to_string());
    
    let db_pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");
    
    // Connect to Valkey/Redis (still used for WebSocket broadcasting)
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://default:HpNKUsNN27031968@www.goyav.re:6379/".to_string());
    
    let redis_client = Client::open(redis_url).expect("Failed to create Redis client");
    let redis_connection = ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");
    
    // Create broadcast channel for WebSocket messages
    let (broadcast_tx, _) = broadcast::channel(100);
    
    let app_state = AppState {
        db: db_pool,
        redis: redis_connection,
        broadcast_tx,
    };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/users", get(get_users).post(create_user))
        .route("/users/:id", get(get_user).delete(delete_user))
        .route("/health", get(health_check))
        .route("/cache/:key", get(get_cache).post(set_cache).delete(delete_cache))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(ServiceBuilder::new())
        .with_state(app_state);

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Server running on http://0.0.0.0:3000");
    println!("WebSocket available at ws://0.0.0.0:3000/ws");
    println!("Test page available at http://0.0.0.0:3000/static/index.html");
    println!("PostgreSQL database connected");
    println!("Redis/Valkey connected for WebSocket broadcasting");
    
    axum::serve(listener, app).await.unwrap();
}

// Handler functions
async fn hello_world(Query(params): Query<Params>) -> &'static str {
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

async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Test Redis connection with a simple command
    let mut conn = state.redis.clone();
    let redis_status = match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };
    
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "redis": redis_status
    }))
}

async fn get_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, StatusCode> {
    match sqlx::query_as::<_, User>("SELECT id, name, email, created_at, updated_at FROM users ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await 
    {
        Ok(users) => Ok(Json(users)),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user(axum::extract::Path(id): axum::extract::Path<i32>, State(state): State<AppState>) -> Result<Json<User>, StatusCode> {
    match sqlx::query_as::<_, User>("SELECT id, name, email, created_at, updated_at FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await 
    {
        Ok(user) => Ok(Json(user)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_user(State(state): State<AppState>, Json(payload): Json<CreateUser>) -> Result<Json<User>, StatusCode> {
    // Insert user into PostgreSQL
    match sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email, created_at, updated_at"
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await 
    {
        Ok(user) => {
            // Create notification for WebSocket
            let notification = UserNotification {
                id: Uuid::new_v4().to_string(),
                event_type: "user_created".to_string(),
                user_data: user.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                message: format!("Nouvel utilisateur créé: {} ({})", user.name, user.email),
            };
            
            // Store notification in database
            let _ = sqlx::query(
                "INSERT INTO user_events (event_type, user_id, user_data, message) VALUES ($1, $2, $3, $4)"
            )
            .bind(&notification.event_type)
            .bind(user.id)
            .bind(serde_json::to_value(&user).unwrap_or_default())
            .bind(&notification.message)
            .execute(&state.db)
            .await;
            
            // Send notification via WebSocket
            if let Ok(notification_json) = serde_json::to_string(&notification) {
                let _ = state.broadcast_tx.send(notification_json);
            }
            
            Ok(Json(user))
        }
        Err(sqlx::Error::Database(db_err)) if db_err.constraint() == Some("users_email_key") => {
            Err(StatusCode::CONFLICT) // Email already exists
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_user(axum::extract::Path(id): axum::extract::Path<i32>, State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    // Get user data before deletion for notification
    let user_data = sqlx::query_as::<_, User>("SELECT id, name, email, created_at, updated_at FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Delete user from database
    match sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await 
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // Send notification if we have user data
                if let Some(user) = user_data {
                    let notification = UserNotification {
                        id: Uuid::new_v4().to_string(),
                        event_type: "user_deleted".to_string(),
                        user_data: user.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        message: format!("Utilisateur supprimé: {} ({})", user.name, user.email),
                    };
                    
                    // Store notification in database
                    let _ = sqlx::query(
                        "INSERT INTO user_events (event_type, user_id, user_data, message) VALUES ($1, $2, $3, $4)"
                    )
                    .bind(&notification.event_type)
                    .bind(user.id)
                    .bind(serde_json::to_value(&user).unwrap_or_default())
                    .bind(&notification.message)
                    .execute(&state.db)
                    .await;
                    
                    // Send notification via WebSocket
                    if let Ok(notification_json) = serde_json::to_string(&notification) {
                        let _ = state.broadcast_tx.send(notification_json);
                    }
                }
                
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Cache endpoints for generic key-value storage
async fn get_cache(axum::extract::Path(key): axum::extract::Path<String>, State(state): State<AppState>) -> Result<String, StatusCode> {
    let mut conn = state.redis.clone();
    
    match redis::cmd("GET")
        .arg(&key)
        .query_async::<_, String>(&mut conn)
        .await 
    {
        Ok(value) => Ok(value),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Deserialize)]
struct CacheValue {
    value: String,
    ttl: Option<u64>, // Time to live in seconds
}

async fn set_cache(axum::extract::Path(key): axum::extract::Path<String>, State(state): State<AppState>, Json(payload): Json<CacheValue>) -> Result<StatusCode, StatusCode> {
    let mut conn = state.redis.clone();
    
    let result = if let Some(ttl) = payload.ttl {
        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl)
            .arg(&payload.value)
            .query_async::<_, ()>(&mut conn)
            .await
    } else {
        redis::cmd("SET")
            .arg(&key)
            .arg(&payload.value)
            .query_async::<_, ()>(&mut conn)
            .await
    };
    
    match result {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_cache(axum::extract::Path(key): axum::extract::Path<String>, State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    let mut conn = state.redis.clone();
    
    match redis::cmd("DEL")
        .arg(&key)
        .query_async::<_, i32>(&mut conn)
        .await 
    {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

async fn websocket_connection(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.broadcast_tx.subscribe();
    
    // Spawn a task to handle incoming messages from this WebSocket connection
    let broadcast_tx = state.broadcast_tx.clone();
    let mut redis_conn = state.redis.clone();
    
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        println!("Received message: {}", text);
                        
                        // Try to parse as JSON message
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            // Store message in Redis
                            let key = format!("message:{}", ws_msg.id);
                            if let Ok(msg_json) = serde_json::to_string(&ws_msg) {
                                let _: Result<(), _> = redis::cmd("SET")
                                    .arg(&key)
                                    .arg(&msg_json)
                                    .arg("EX")
                                    .arg(3600) // Expire after 1 hour
                                    .query_async(&mut redis_conn)
                                    .await;
                            }
                            
                            // Broadcast message to all connected clients
                            let _ = broadcast_tx.send(text);
                        } else {
                            // Simple text message - create a WsMessage structure
                            let ws_msg = WsMessage {
                                id: Uuid::new_v4().to_string(),
                                user: "anonymous".to_string(),
                                message: text,
                                timestamp: chrono::Utc::now().to_rfc3339(),
                            };
                            
                            if let Ok(msg_json) = serde_json::to_string(&ws_msg) {
                                // Store in Redis
                                let key = format!("message:{}", ws_msg.id);
                                let _: Result<(), _> = redis::cmd("SET")
                                    .arg(&key)
                                    .arg(&msg_json)
                                    .arg("EX")
                                    .arg(3600)
                                    .query_async(&mut redis_conn)
                                    .await;
                                
                                // Broadcast to all clients
                                let _ = broadcast_tx.send(msg_json);
                            }
                        }
                    }
                    Message::Binary(_) => {
                        println!("Received binary message");
                    }
                    Message::Close(_) => {
                        println!("WebSocket connection closed");
                        break;
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }
    });
    
    // Spawn a task to handle outgoing messages to this WebSocket connection
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
