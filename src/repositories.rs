use async_trait::async_trait;
use sqlx::PgPool;
use redis::aio::ConnectionManager;
use crate::models::{User, CreateUserRequest, CacheValue, UserNotification};
use crate::errors::{AppError, Result};

// User Repository Interface (Interface Segregation Principle)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<User>>;
    async fn find_by_id(&self, id: i32) -> Result<Option<User>>;
    async fn create(&self, request: CreateUserRequest) -> Result<User>;
    async fn delete(&self, id: i32) -> Result<Option<User>>;
}

// Cache Repository Interface
#[async_trait]
pub trait CacheRepository: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &CacheValue) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<bool>;
}

// Event Repository Interface
#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn store_user_event(&self, notification: &UserNotification) -> Result<()>;
}

// PostgreSQL Implementation
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_all(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, name, email, created_at, updated_at FROM users ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)?;
        
        Ok(users)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, name, email, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;
        
        Ok(user)
    }

    async fn create(&self, request: CreateUserRequest) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email, created_at, updated_at"
        )
        .bind(&request.name)
        .bind(&request.email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.constraint() == Some("users_email_key") => {
                AppError::EmailConflict
            }
            _ => AppError::Database(e),
        })?;
        
        Ok(user)
    }

    async fn delete(&self, id: i32) -> Result<Option<User>> {
        // Get user data before deletion
        let user = self.find_by_id(id).await?;
        
        if user.is_some() {
            let result = sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(AppError::Database)?;
            
            if result.rows_affected() > 0 {
                Ok(user)
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

// Redis Cache Implementation
pub struct RedisCacheRepository {
    redis: ConnectionManager,
}

impl RedisCacheRepository {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }
}

#[async_trait]
impl CacheRepository for RedisCacheRepository {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.redis.clone();
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(AppError::Redis)?;
        
        Ok(result)
    }

    async fn set(&self, key: &str, value: &CacheValue) -> Result<()> {
        let mut conn = self.redis.clone();
        
        if let Some(ttl) = value.ttl {
            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl)
                .arg(&value.value)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(AppError::Redis)?;
        } else {
            redis::cmd("SET")
                .arg(key)
                .arg(&value.value)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(AppError::Redis)?;
        }
        
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let mut conn = self.redis.clone();
        let deleted: i32 = redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(AppError::Redis)?;
        
        Ok(deleted > 0)
    }
}

// PostgreSQL Event Repository
pub struct PostgresEventRepository {
    pool: PgPool,
}

impl PostgresEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventRepository for PostgresEventRepository {
    async fn store_user_event(&self, notification: &UserNotification) -> Result<()> {
        let _ = sqlx::query(
            "INSERT INTO user_events (event_type, user_id, user_data, message) VALUES ($1, $2, $3, $4)"
        )
        .bind(&notification.event_type)
        .bind(notification.user_data.id)
        .bind(serde_json::to_value(&notification.user_data).unwrap_or_default())
        .bind(&notification.message)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;
        
        Ok(())
    }
}
