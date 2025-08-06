use sqlx::PgPool;
use redis::aio::ConnectionManager;
use crate::config::Config;
use crate::errors::{AppError, Result};

pub struct DatabaseConnections {
    pub pg_pool: PgPool,
    pub redis: ConnectionManager,
}

impl DatabaseConnections {
    pub async fn new(config: &Config) -> Result<Self> {
        let pg_pool = PgPool::connect(&config.database.url)
            .await
            .map_err(AppError::Database)?;

        // Run migrations
        if let Err(e) = sqlx::migrate!("./migrations").run(&pg_pool).await {
            eprintln!("Migration error: {}", e);
            return Err(AppError::Internal);
        }

        let redis_client = redis::Client::open(config.redis.url.clone())
            .map_err(AppError::Redis)?;
        
        let redis = ConnectionManager::new(redis_client)
            .await
            .map_err(AppError::Redis)?;

        Ok(Self { pg_pool, redis })
    }

    pub fn pg_pool(&self) -> &PgPool {
        &self.pg_pool
    }

    pub fn redis(&self) -> &ConnectionManager {
        &self.redis
    }
}
