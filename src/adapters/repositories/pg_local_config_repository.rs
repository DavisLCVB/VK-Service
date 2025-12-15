use async_trait::async_trait;
use sqlx::{query_as, QueryBuilder};
use tracing::{debug, info};

use crate::{
    application::{
        dto::local_config_dto::LocalConfigDTO, error::ApplicationError,
        repositories::local_config_repository::LocalConfigRepository,
    },
    domain::config::local::{LocalConfig, Provider},
};

pub struct PgLocalConfigRepository {
    pool: sqlx::PgPool,
}

impl PgLocalConfigRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LocalConfigRepository for PgLocalConfigRepository {
    async fn get_local_config(&self, server_id: &str) -> Result<LocalConfig, ApplicationError> {
        let query = "SELECT * FROM config.local WHERE server_id = $1";
        let config_dto: LocalConfigDTO = query_as::<_, LocalConfigDTO>(query)
            .bind(server_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => ApplicationError::NotFound,
                _ => ApplicationError::DatabaseError(e.to_string()),
            })?;

        // server_id must be set manually after DTO conversion
        let mut config: LocalConfig = config_dto.into();
        config.server_id = server_id.to_string();
        Ok(config)
    }

    async fn upsert_local_config(
        &self,
        server_id: &str,
        config: LocalConfigDTO,
    ) -> Result<LocalConfig, ApplicationError> {
        debug!("Upserting local config for server_id: {}", server_id);
        let mut config = config;
        config.sanitize();

        // If no fields provided, insert with defaults or get existing
        if config.provider.is_none() && config.server_name.is_none() && config.server_url.is_none()
        {
            debug!(
                "No fields provided, inserting default config or getting existing for server_id: {}",
                server_id
            );

            // Try to insert with defaults, or return existing if conflict
            let query = "
                INSERT INTO config.local (server_id, provider, server_name, server_url)
                VALUES ($1, 'gdrive', '', '')
                ON CONFLICT (server_id) DO UPDATE SET server_id = EXCLUDED.server_id
                RETURNING *
            ";

            let config_dto: LocalConfigDTO = query_as::<_, LocalConfigDTO>(query)
                .bind(server_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

            let mut result: LocalConfig = config_dto.into();
            result.server_id = server_id.to_string();
            return Ok(result);
        }

        // Check if record exists to determine INSERT or UPDATE
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM config.local WHERE server_id = $1)"
        )
        .bind(server_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        let updated_config_dto: LocalConfigDTO = if exists {
            // UPDATE existing record with only provided fields
            let mut builder: QueryBuilder<sqlx::Postgres> =
                QueryBuilder::new("UPDATE config.local SET ");
            let mut separated = builder.separated(", ");

            if let Some(provider) = &config.provider {
                let provider_str = match provider {
                    Provider::GDrive => "gdrive",
                    Provider::Supabase => "supabase",
                };
                separated.push("provider = ");
                separated.push_bind_unseparated(provider_str);
            }

            if let Some(server_name) = &config.server_name {
                separated.push("server_name = ");
                separated.push_bind_unseparated(server_name);
            }

            if let Some(server_url) = &config.server_url {
                separated.push("server_url = ");
                separated.push_bind_unseparated(server_url);
            }

            builder.push(" WHERE server_id = ");
            builder.push_bind(server_id);
            builder.push(" RETURNING *");

            builder
                .build_query_as::<LocalConfigDTO>()
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?
        } else {
            // INSERT new record - need all fields, use defaults for missing ones
            let provider_str = match config.provider {
                Some(Provider::GDrive) => "gdrive",
                Some(Provider::Supabase) => "supabase",
                None => "gdrive", // default
            };
            let server_name = config.server_name.as_deref().unwrap_or("");
            let server_url = config.server_url.as_deref().unwrap_or("");

            query_as::<_, LocalConfigDTO>(
                "INSERT INTO config.local (server_id, provider, server_name, server_url)
                 VALUES ($1, $2, $3, $4)
                 RETURNING *"
            )
            .bind(server_id)
            .bind(provider_str)
            .bind(server_name)
            .bind(server_url)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?
        };

        // server_id must be set manually after DTO conversion
        let mut result: LocalConfig = updated_config_dto.into();
        result.server_id = server_id.to_string();
        info!(
            "Local config upserted successfully for server_id: {}, provider: {:?}, server_name: {}",
            server_id, result.provider, result.server_name
        );
        Ok(result)
    }

    async fn get_all_instance_ids(&self) -> Result<Vec<String>, ApplicationError> {
        let query = "SELECT server_id FROM config.local ORDER BY server_id";
        let rows: Vec<(String,)> = sqlx::query_as(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}
