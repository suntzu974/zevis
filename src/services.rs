use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::broadcast;
use crate::models::{User, CreateUserRequest, CacheValue, UserNotification};
use crate::repositories::{UserRepository, CacheRepository, EventRepository};
use crate::errors::{AppError, Result};

// Service Interfaces (Interface Segregation Principle)
#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_all_users(&self) -> Result<Vec<User>>;
    async fn get_user_by_id(&self, id: i32) -> Result<User>;
    async fn get_user_by_email(&self, email: &str) -> Result<User>;
    async fn create_user(&self, request: CreateUserRequest) -> Result<User>;
    async fn create_user_with_password(&self, user: User) -> Result<User>;
    async fn delete_user(&self, id: i32) -> Result<()>;
}

#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get_cache_value(&self, key: &str) -> Result<String>;
    async fn set_cache_value(&self, key: &str, value: CacheValue) -> Result<()>;
    async fn delete_cache_value(&self, key: &str) -> Result<()>;
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn notify_user_created(&self, user: &User) -> Result<()>;
    async fn notify_user_deleted(&self, user: &User) -> Result<()>;
}

// User Service Implementation
pub struct UserServiceImpl {
    user_repo: Arc<dyn UserRepository>,
    event_repo: Arc<dyn EventRepository>,
    notification_service: Arc<dyn NotificationService>,
}

impl UserServiceImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        event_repo: Arc<dyn EventRepository>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            user_repo,
            event_repo,
            notification_service,
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_all_users(&self) -> Result<Vec<User>> {
        self.user_repo.find_all().await
    }

    async fn get_user_by_id(&self, id: i32) -> Result<User> {
        match self.user_repo.find_by_id(id).await? {
            Some(user) => Ok(user),
            None => Err(AppError::UserNotFound),
        }
    }

    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        match self.user_repo.find_by_email(email).await? {
            Some(user) => Ok(user),
            None => Err(AppError::UserNotFound),
        }
    }

    async fn create_user(&self, request: CreateUserRequest) -> Result<User> {
        let user = self.user_repo.create(request).await?;
        
        // Notify about user creation
        if let Err(e) = self.notification_service.notify_user_created(&user).await {
            eprintln!("Failed to send notification: {}", e);
        }
        
        Ok(user)
    }

    async fn create_user_with_password(&self, user: User) -> Result<User> {
        let created_user = self.user_repo.create_with_password(user).await?;
        
        // Notify about user creation
        if let Err(e) = self.notification_service.notify_user_created(&created_user).await {
            eprintln!("Failed to send notification: {}", e);
        }
        
        Ok(created_user)
    }

    async fn delete_user(&self, id: i32) -> Result<()> {
        match self.user_repo.delete(id).await? {
            Some(user) => {
                // Notify about user deletion
                if let Err(e) = self.notification_service.notify_user_deleted(&user).await {
                    eprintln!("Failed to send notification: {}", e);
                }
                Ok(())
            }
            None => Err(AppError::UserNotFound),
        }
    }
}

// Cache Service Implementation
pub struct CacheServiceImpl {
    cache_repo: Arc<dyn CacheRepository>,
}

impl CacheServiceImpl {
    pub fn new(cache_repo: Arc<dyn CacheRepository>) -> Self {
        Self { cache_repo }
    }
}

#[async_trait]
impl CacheService for CacheServiceImpl {
    async fn get_cache_value(&self, key: &str) -> Result<String> {
        match self.cache_repo.get(key).await? {
            Some(value) => Ok(value),
            None => Err(AppError::CacheKeyNotFound),
        }
    }

    async fn set_cache_value(&self, key: &str, value: CacheValue) -> Result<()> {
        self.cache_repo.set(key, &value).await
    }

    async fn delete_cache_value(&self, key: &str) -> Result<()> {
        if !self.cache_repo.delete(key).await? {
            return Err(AppError::CacheKeyNotFound);
        }
        Ok(())
    }
}

// Notification Service Implementation
pub struct NotificationServiceImpl {
    event_repo: Arc<dyn EventRepository>,
    broadcast_tx: broadcast::Sender<String>,
}

impl NotificationServiceImpl {
    pub fn new(
        event_repo: Arc<dyn EventRepository>,
        broadcast_tx: broadcast::Sender<String>,
    ) -> Self {
        Self {
            event_repo,
            broadcast_tx,
        }
    }

    async fn send_notification(&self, notification: UserNotification) -> Result<()> {
        // Store event in database
        self.event_repo.store_user_event(&notification).await?;
        
        // Broadcast via WebSocket
        if let Ok(notification_json) = serde_json::to_string(&notification) {
            let _ = self.broadcast_tx.send(notification_json);
        }
        
        Ok(())
    }
}

#[async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn notify_user_created(&self, user: &User) -> Result<()> {
        let notification = UserNotification::new_created(user.clone());
        self.send_notification(notification).await
    }

    async fn notify_user_deleted(&self, user: &User) -> Result<()> {
        let notification = UserNotification::new_deleted(user.clone());
        self.send_notification(notification).await
    }
}
