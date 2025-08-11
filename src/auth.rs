use std::time::{SystemTime, Duration, UNIX_EPOCH};
use axum::{http::{header}, response::IntoResponse, http::Request};
use axum::middleware::Next;
use axum::extract::State;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, TokenData, errors::ErrorKind};
use serde::{Serialize, Deserialize};
use crate::errors::ProblemDetails;
use crate::handlers::AppState;
use jsonwebtoken::{encode, EncodingKey, Header};
use dashmap::DashMap;
use std::net::IpAddr;
use once_cell::sync::Lazy;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: Option<String>,
    pub scope: Option<String>,
}

impl Claims {
    pub fn new(sub: impl Into<String>, ttl: Duration, issuer: Option<String>) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs() as usize;
        Self { sub: sub.into(), exp: now + ttl.as_secs() as usize, iat: now, iss: issuer, scope: None }
    }
}

pub async fn jwt_middleware(State(state): State<AppState>, mut req: Request<axum::body::Body>, next: Next) -> Result<axum::response::Response, axum::response::Response> {
    let auth_header_val = req.headers().get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let token = auth_header_val.strip_prefix("Bearer ")
        .ok_or_else(|| ProblemDetails::new("about:blank", "Unauthorized", 401)
            .with_detail("Missing or invalid Authorization header").into_response())?;

    let key = DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    if let Some(ref iss) = state.jwt_issuer { validation.set_issuer(&[iss]); }

    let claims = match decode::<Claims>(token, &key, &validation) {
        Ok(TokenData { claims, .. }) => claims,
        Err(e) => {
            let status = match e.kind() { ErrorKind::ExpiredSignature => 401, _ => 401 };
            let pd = ProblemDetails::new("about:blank", "Unauthorized", status)
                .with_detail(&format!("Invalid token: {}", e));
            return Err(pd.into_response());
        }
    };

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub fn encode_token(sub: &str, ttl: Duration, secret: &str, issuer: Option<&str>) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(sub.to_string(), ttl, issuer.map(|s| s.to_string()));
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

// Simple IP-based rate limiter (fixed window)
#[derive(Clone, Default)]
pub struct RateLimiter {
    // key: ip, value: (window_start_millis, count)
    buckets: std::sync::Arc<DashMap<IpAddr, (u128, u32)>>,
    pub window_ms: u128,
    pub max: u32,
}

impl RateLimiter {
    pub fn new(window: Duration, max: u32) -> Self {
        Self { buckets: Default::default(), window_ms: window.as_millis(), max }
    }
}

pub async fn rate_limit_middleware(State(_state): State<AppState>, req: Request<axum::body::Body>, next: Next) -> axum::response::Response {
    static RL: Lazy<RateLimiter> = Lazy::new(|| RateLimiter::new(Duration::from_secs(1), 200));
    let ip = req.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip())
        .unwrap_or(std::net::IpAddr::from([127,0,0,1]));
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_millis();
    let mut allow = false;
    {
        let mut entry = RL.buckets.entry(ip).or_insert((now, 0));
        let (start, count) = *entry;
        if now - start >= RL.window_ms {
            *entry = (now, 1);
            allow = true;
        } else if count < RL.max {
            *entry = (start, count + 1);
            allow = true;
        }
    }
    if !allow {
        return ProblemDetails::new("about:blank", "Too Many Requests", 429).with_detail("Rate limit exceeded").into_response();
    }
    next.run(req).await
}
