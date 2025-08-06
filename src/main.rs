use std::sync::Arc;
use axum::{
    routing::{get, post, delete},
    Router,
};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

// Import our modules
use zevis::{
    config::Config,
    database::DatabaseConnections,
    handlers::{self, AppState},
    repositories::{PostgresUserRepository, RedisCacheRepository, PostgresEventRepository},
    services::{UserServiceImpl, CacheServiceImpl, NotificationServiceImpl},
    websocket::websocket_handler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize database connections
    let db_connections = DatabaseConnections::new(&config).await?;
    
    // Create broadcast channel for WebSocket messages
    let (broadcast_tx, _) = broadcast::channel(100);
    
    // Initialize repositories (Dependency Injection)
    let user_repo = Arc::new(PostgresUserRepository::new(db_connections.pg_pool().clone()));
    let cache_repo = Arc::new(RedisCacheRepository::new(db_connections.redis().clone()));
    let event_repo = Arc::new(PostgresEventRepository::new(db_connections.pg_pool().clone()));
    
    // Initialize services (Dependency Injection)
    let notification_service = Arc::new(NotificationServiceImpl::new(
        event_repo.clone(),
        broadcast_tx.clone(),
    ));
    
    let user_service = Arc::new(UserServiceImpl::new(
        user_repo,
        event_repo,
        notification_service,
    ));
    
    let cache_service = Arc::new(CacheServiceImpl::new(cache_repo));
    
    // Create unified application state
    let app_state = AppState {
        user_service,
        cache_service,
        broadcast_tx,
    };
    
    // Build router
    let app = Router::new()
        .route("/", get(handlers::hello_world))
        .route("/users", get(handlers::get_users).post(handlers::create_user))
        .route("/users/:id", get(handlers::get_user).delete(handlers::delete_user))
        .route("/health", get(handlers::health_check))
        .route("/cache/:key", 
            get(handlers::get_cache)
                .post(handlers::set_cache)
                .delete(handlers::delete_cache)
        )
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/notifications", ServeDir::new("yew-ws/dist")) // Yew WebSocket notifications frontend
        .nest_service("/react", ServeDir::new("react-ws/build")) // React WebSocket notifications frontend
        .layer(ServiceBuilder::new())
        .with_state(app_state);
    
    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("🚀 Server running on http://{}", addr);
    println!("📡 WebSocket available at ws://{}/ws", addr);
    println!("🌐 Test page available at http://{}/static/index.html", addr);
    println!("🦀 Yew WebSocket notifications frontend at http://{}/notifications/", addr);
    println!("⚛️ React WebSocket notifications frontend at http://{}/react/", addr);
    println!("🗄️ PostgreSQL database connected");
    println!("🔄 Redis connected for WebSocket broadcasting");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
