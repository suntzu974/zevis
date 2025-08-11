use axum::http::{HeaderName, Method};
use std::time::Duration;
use tower_http::cors::CorsLayer;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        // Allow specific origins in production
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "http://127.0.0.1:3000".parse().unwrap(),
        ])
        // Allow specific methods
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        // Allow specific headers
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("x-requested-with"),
        ])
        // Allow credentials for authentication
        .allow_credentials(true)
        // Cache preflight requests for 1 hour
        .max_age(Duration::from_secs(3600))
}
