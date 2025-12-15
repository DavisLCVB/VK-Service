use async_trait::async_trait;
use sqlx::{query_as, QueryBuilder};
use tracing::{debug, info};

use crate::{
    application::{
        dto::global_config_dto::GlobalConfigDTO, error::ApplicationError,
        repositories::global_config_repository::GlobalConfigRepository,
    },
    domain::config::global::GlobalConfig,
};

pub struct PgGlobalConfigRepository {
    pool: sqlx::PgPool,
}

impl PgGlobalConfigRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GlobalConfigRepository for PgGlobalConfigRepository {
    async fn get_global_config(&self) -> Result<GlobalConfig, ApplicationError> {
        debug!("Fetching global config from database");
        let query = "SELECT * FROM config.global LIMIT 1";
        let config_dto: GlobalConfigDTO = query_as::<_, GlobalConfigDTO>(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;
        let config: GlobalConfig = config_dto.into();
        info!(
            "Global config fetched successfully: max_size={}, default_quota={}",
            config.max_size, config.default_quota
        );
        Ok(config)
    }

    async fn upsert_global_config(
        &self,
        config: GlobalConfigDTO,
    ) -> Result<GlobalConfig, ApplicationError> {
        let mut config = config;
        config.sanitize();

        // If all fields are None, just return the current config
        if config.mime_types.is_none()
            && config.max_size.is_none()
            && config.chunk_size.is_none()
            && config.temp_file_life.is_none()
            && config.default_quota.is_none()
        {
            return self.get_global_config().await;
        }

        // Dynamic UPDATE query building for single-row table
        let mut builder: QueryBuilder<sqlx::Postgres> =
            QueryBuilder::new("UPDATE config.global SET ");
        let mut separated = builder.separated(", ");

        if let Some(mime_types) = &config.mime_types {
            separated.push("mime_types = ");
            separated.push_bind_unseparated(mime_types);
        }

        if let Some(max_size) = config.max_size {
            separated.push("max_size = ");
            separated.push_bind_unseparated(max_size as i64);
        }

        if let Some(chunk_size) = config.chunk_size {
            separated.push("chunk_size = ");
            separated.push_bind_unseparated(chunk_size as i64);
        }

        if let Some(temp_file_life) = config.temp_file_life {
            separated.push("temp_file_life = ");
            separated.push_bind_unseparated(temp_file_life as i64);
        }

        if let Some(default_quota) = config.default_quota {
            separated.push("default_quota = ");
            separated.push_bind_unseparated(default_quota as i64);
        }

        builder.push(" RETURNING *");

        let query = builder.build_query_as::<GlobalConfigDTO>();
        let updated_config_dto: GlobalConfigDTO = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(updated_config_dto.into())
    }
}
