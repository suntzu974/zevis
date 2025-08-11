use std::sync::Arc;
use axum::{
    routing::{get, post, delete},
    Router,
};
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};

// Import our modules
use zevis::{
    config::Config,
    database::DatabaseConnections,
    handlers::{self, AppState},
    repositories::{PostgresUserRepository, RedisCacheRepository, PostgresEventRepository},
    services::{UserServiceImpl, CacheServiceImpl, NotificationServiceImpl},
    websocket::websocket_handler,
    middleware::{cors_layer},
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
    
    let static_files = ServeDir::new("./public");
    
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/", get(handlers::hello_world))
        .route("/health", get(handlers::health_check))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"));

    // Authentication routes
    let auth_routes = Router::new()
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        .with_state(app_state.clone());

    // Protected routes (require authentication)
    let protected_routes = Router::new()
        .route("/auth/me", get(handlers::auth::me))
        .route("/auth/protected", get(handlers::auth::protected))
        .route("/users", get(handlers::get_users).post(handlers::create_user))
        .route("/users/{id}", get(handlers::get_user).delete(handlers::delete_user))
        .route("/cache/{key}", 
            get(handlers::get_cache)
                .post(handlers::set_cache)
                .delete(handlers::delete_cache)
        )
        .with_state(app_state.clone());

    // Build main router with all middlewares
    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(protected_routes)
        .fallback_service(
            static_files
                .clone()
                .not_found_service(ServeFile::new("./public/index.html"))
        )
        .layer(
            ServiceBuilder::new()
                .layer(cors_layer()) // CORS support
        )
        .with_state(app_state);
    
    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("🚀 Server running on http://{}", addr);
    println!("📡 WebSocket available at ws://{}/ws", addr);
    println!("🌐 Test page available at http://{}/static/index.html", addr);
    println!("⚛️ React WebSocket notifications frontend at http://{}/react/", addr);
    println!("🦀 Yew WebSocket notifications frontend at http://{}/yew/", addr);
    println!("� Authentication endpoints:");
    println!("   - POST http://{}/auth/register", addr);
    println!("   - POST http://{}/auth/login", addr);
    println!("   - GET http://{}/auth/me (requires JWT)", addr);
    println!("   - GET http://{}/auth/protected (requires JWT)", addr);
    println!("�🗄️ PostgreSQL database connected");
    println!("🔄 Redis connected for WebSocket broadcasting");
    println!("🛡️ CORS and rate limiting enabled");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
