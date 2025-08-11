use std::sync::Arc;
use axum::{
    routing::{get, post, delete},
    Router,
};
use tokio::sync::broadcast;
use tower_http::cors::{CorsLayer, AllowOrigin};
use axum::http;
use tower_http::services::{ServeDir, ServeFile};

// Import our modules
use zevis::{
    config::Config,
    database::DatabaseConnections,
    handlers::{self, AppState},
    repositories::{PostgresUserRepository, RedisCacheRepository, PostgresEventRepository},
    services::{UserServiceImpl, CacheServiceImpl, NotificationServiceImpl},
    websocket::websocket_handler,
    auth::{self, encode_token},
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
    jwt_secret: config.auth.jwt_secret.clone(),
    jwt_issuer: config.auth.jwt_issuer.clone(),
    };
    
    let static_files = ServeDir::new("./public");

    // CORS: strict allow-list from config
    let allowed_origins = AllowOrigin::list(
        config
            .auth
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect::<Vec<http::HeaderValue>>()
    );
    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([http::Method::GET, http::Method::POST, http::Method::DELETE])
        .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION]);
    // Build router
    // Public routes
    let public = Router::new()
        .route("/", get(handlers::hello_world))
        .route("/auth/register", post(handlers::register_user))
        .route("/auth/login", post(handlers::login))
        .route("/health", get(handlers::health_check))
        .route("/ws", get(websocket_handler))
        .nest_service("/static", ServeDir::new("static"));

    // Protected routes
    let protected = Router::new()
        .route("/users", get(handlers::get_users).post(handlers::create_user))
        .route("/users/{id}", get(handlers::get_user).delete(handlers::delete_user))
        .route("/cache/{key}", 
            get(handlers::get_cache)
                .post(handlers::set_cache)
                .delete(handlers::delete_cache)
        )
        .route_layer(axum::middleware::from_fn_with_state(app_state.clone(), auth::jwt_middleware));

    // Rate limiting middleware applied later at router level

    let app = Router::new()
        .merge(public)
        .merge(protected)
        // .fallback_service(
        //    static_files
        //        .clone()
        //        .not_found_service(ServeFile::new("./public/index.html")), ) // Yew WebSocket notifications frontend with SPA fallback
    .layer(cors.clone())
    .route_layer(axum::middleware::from_fn_with_state(app_state.clone(), auth::rate_limit_middleware))
        .with_state(app_state);
    
    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("🚀 Server running on http://{}", addr);
    println!("📡 WebSocket available at ws://{}/ws", addr);
    println!("🌐 Test page available at http://{}/static/index.html", addr);
    println!("⚛️ React WebSocket notifications frontend at http://{}/react/", addr);
    println!("🦀 Yew WebSocket notifications frontend at http://{}/yew/", addr);
    println!("🗄️ PostgreSQL database connected");
    println!("🔄 Redis connected for WebSocket broadcasting");
    
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
    
    Ok(())
}
