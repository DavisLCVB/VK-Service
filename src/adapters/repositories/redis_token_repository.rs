use async_trait::async_trait;
use redis::AsyncCommands;
use tracing::info;
use uuid::Uuid;

use crate::application::{
    error::ApplicationError, repositories::token_repository::TokenRepository,
};

pub struct RedisTokenRepository {
    client: redis::aio::ConnectionManager,
}

impl RedisTokenRepository {
    pub fn new(client: redis::aio::ConnectionManager) -> Self {
        Self { client }
    }

    fn get_redis_key(token: &str) -> String {
        format!("upload_token:{}", token)
    }
}

#[async_trait]
impl TokenRepository for RedisTokenRepository {
    async fn generate_token(
        &self,
        user_id: Option<String>,
        ttl_seconds: u64,
    ) -> Result<String, ApplicationError> {
        let token = Uuid::new_v4().to_string();
        let key = Self::get_redis_key(&token);
        let value = user_id.clone().unwrap_or_default();

        info!(
            "Storing token in Redis: key='{}', value='{}', user_id={:?}",
            key, value, user_id
        );

        let mut conn = self.client.clone();

        conn.set_ex::<_, _, ()>(&key, &value, ttl_seconds)
            .await
            .map_err(|e| {
                ApplicationError::InternalError(format!("Failed to store token: {}", e))
            })?;

        info!("Token stored successfully in Redis");
        Ok(token)
    }

    async fn verify_and_consume_token(
        &self,
        token: &str,
    ) -> Result<Option<String>, ApplicationError> {
        let key = Self::get_redis_key(token);
        let mut conn = self.client.clone();

        info!("Verifying and consuming token from Redis: key='{}'", key);

        // GETDEL es atómico - garantiza un solo uso
        let value: Option<String> = conn.get_del(&key).await.map_err(|e| {
            ApplicationError::InternalError(format!("Failed to verify token: {}", e))
        })?;

        info!("Token value retrieved from Redis: {:?}", value);

        match value {
            None => {
                info!("Token not found or already consumed");
                Err(ApplicationError::InvalidToken)
            }
            Some(v) if v.is_empty() => {
                info!("Token is anonymous (empty value)");
                Ok(None)
            }
            Some(user_id) => {
                info!("Token associated with user_id: {}", user_id);
                Ok(Some(user_id))
            }
        }
    }
}
