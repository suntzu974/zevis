use axum::{
    extract::{Query, State, WebSocketUpgrade, ws::{WebSocket, Message}},
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post, delete},
    Router,
};
use redis::{aio::ConnectionManager, Client};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use uuid::Uuid;
use tokio::sync::broadcast;
use futures_util::{SinkExt, StreamExt};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
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

// Application state containing Valkey connection and broadcast channel
#[derive(Clone)]
struct AppState {
    redis: ConnectionManager,
    broadcast_tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    // Connect to Valkey/Redis
    let redis_client = Client::open("redis://default:HpNKUsNN27031968@www.goyav.re:6379/").expect("Failed to create Redis client");
    let redis_connection = ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");
    
    // Create broadcast channel for WebSocket messages
    let (broadcast_tx, _) = broadcast::channel(100);
    
    let app_state = AppState {
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("Server running on http://127.0.0.1:3000");
    println!("WebSocket available at ws://127.0.0.1:3000/ws");
    println!("Test page available at http://127.0.0.1:3000/static/index.html");
    println!("Make sure Valkey/Redis is running on www.goyav.re:6379");
    
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
    let mut conn = state.redis.clone();
    
    // Get all user keys
    let keys: Vec<String> = match redis::cmd("KEYS")
        .arg("user:*")
        .query_async(&mut conn)
        .await 
    {
        Ok(keys) => keys,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    let mut users = Vec::new();
    
    for key in keys {
        if let Ok(user_json) = redis::cmd("GET")
            .arg(&key)
            .query_async::<_, String>(&mut conn)
            .await 
        {
            if let Ok(user) = serde_json::from_str::<User>(&user_json) {
                users.push(user);
            }
        }
    }
    
    // If no users in Redis, return some default users and store them
    if users.is_empty() {
        let default_users = vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ];
        
        // Store default users in Redis
        for user in &default_users {
            if let Ok(user_json) = serde_json::to_string(user) {
                let _: () = redis::cmd("SET")
                    .arg(format!("user:{}", user.id))
                    .arg(user_json)
                    .query_async(&mut conn)
                    .await
                    .unwrap_or(());
            }
        }
        
        return Ok(Json(default_users));
    }
    
    Ok(Json(users))
}

async fn get_user(axum::extract::Path(id): axum::extract::Path<u32>, State(state): State<AppState>) -> Result<Json<User>, StatusCode> {
    let mut conn = state.redis.clone();
    
    match redis::cmd("GET")
        .arg(format!("user:{}", id))
        .query_async::<_, String>(&mut conn)
        .await 
    {
        Ok(user_json) => {
            match serde_json::from_str::<User>(&user_json) {
                Ok(user) => Ok(Json(user)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_user(State(state): State<AppState>, Json(payload): Json<CreateUser>) -> Result<Json<User>, StatusCode> {
    let mut conn = state.redis.clone();
    
    // Generate a new ID (in a real app, you'd want better ID generation)
    let new_id = chrono::Utc::now().timestamp() as u32;
    
    let user = User {
        id: new_id,
        name: payload.name,
        email: payload.email,
    };
    
    // Store user in Redis
    match serde_json::to_string(&user) {
        Ok(user_json) => {
            match redis::cmd("SET")
                .arg(format!("user:{}", user.id))
                .arg(user_json)
                .query_async::<_, ()>(&mut conn)
                .await 
            {
                Ok(_) => Ok(Json(user)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn delete_user(axum::extract::Path(id): axum::extract::Path<u32>, State(state): State<AppState>) -> Result<StatusCode, StatusCode> {
    let mut conn = state.redis.clone();
    
    match redis::cmd("DEL")
        .arg(format!("user:{}", id))
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
