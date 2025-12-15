use async_trait::async_trait;
use sqlx::{query_as, QueryBuilder};
use tracing::{debug, info};

use crate::{
    application::{
        dto::secrets_dto::SecretsDTO, error::ApplicationError,
        repositories::secrets_repository::SecretsRepository,
    },
    domain::config::secrets::Secrets,
};

pub struct PgSecretsRepository {
    pool: sqlx::PgPool,
}

impl PgSecretsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SecretsRepository for PgSecretsRepository {
    async fn get_secrets(&self) -> Result<Secrets, ApplicationError> {
        debug!("Fetching secrets from database");
        let query = "SELECT * FROM config.secrets LIMIT 1";
        let secrets_dto: SecretsDTO = query_as::<_, SecretsDTO>(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;
        let secrets: Secrets = secrets_dto.into();
        info!("Secrets fetched successfully: db_username={}, has_gdrive_secrets={}, has_supabase_secrets={}",
              secrets.db_username,
              secrets.gdrive_secrets.is_some(),
              secrets.supabase_secrets.is_some());
        Ok(secrets)
    }

    async fn upsert_secrets(&self, secrets: SecretsDTO) -> Result<Secrets, ApplicationError> {
        let mut secrets = secrets;
        secrets.sanitize();

        // If all fields are None, just return the current secrets
        if secrets.db_password.is_none()
            && secrets.db_username.is_none()
            && secrets.vk_secret.is_none()
            && secrets.gdrive_secrets.is_none()
            && secrets.supabase_secrets.is_none()
        {
            return self.get_secrets().await;
        }

        // Dynamic UPDATE query building for single-row table
        let mut builder: QueryBuilder<sqlx::Postgres> =
            QueryBuilder::new("UPDATE config.secrets SET ");
        let mut separated = builder.separated(", ");

        if let Some(db_password) = &secrets.db_password {
            separated.push("db_password = ");
            separated.push_bind_unseparated(db_password);
        }

        if let Some(db_username) = &secrets.db_username {
            separated.push("db_username = ");
            separated.push_bind_unseparated(db_username);
        }

        if let Some(vk_secret) = &secrets.vk_secret {
            separated.push("vk_secret = ");
            separated.push_bind_unseparated(vk_secret);
        }

        if let Some(ref gdrive_secrets) = secrets.gdrive_secrets {
            separated.push("gdrive_secrets = ");
            separated.push_bind_unseparated(
                serde_json::to_value(gdrive_secrets).unwrap_or(serde_json::Value::Null),
            );
        }

        if let Some(ref supabase_secrets) = secrets.supabase_secrets {
            separated.push("supabase_secrets = ");
            separated.push_bind_unseparated(
                serde_json::to_value(supabase_secrets).unwrap_or(serde_json::Value::Null),
            );
        }

        builder.push(" RETURNING *");

        let query = builder.build_query_as::<SecretsDTO>();
        let updated_secrets_dto: SecretsDTO = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(updated_secrets_dto.into())
    }
}
