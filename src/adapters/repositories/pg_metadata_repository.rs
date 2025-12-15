use async_trait::async_trait;
use sqlx::{query_as, QueryBuilder};

use crate::{
    application::{
        dto::metadata_dto::MetadataDTO, error::ApplicationError,
        repositories::metadata_repository::MetadataRepository,
    },
    domain::models::metadata::Metadata,
};

pub struct PgMetadataRepository {
    pool: sqlx::PgPool,
}

impl PgMetadataRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MetadataRepository for PgMetadataRepository {
    async fn create_metadata(&self, metadata: MetadataDTO) -> Result<Metadata, ApplicationError> {
        let mut metadata = metadata;
        metadata.sanitize();

        let query = r#"
            INSERT INTO application.metadata (
                file_id, mime_type, size, user_id, description,
                file_name, server_id, uploaded_at, download_count,
                last_access, delete_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
        "#;

        let new_metadata: Metadata = metadata.into();

        let created: MetadataDTO = query_as::<_, MetadataDTO>(query)
            .bind(&new_metadata.file_id)
            .bind(&new_metadata.mime_type)
            .bind(new_metadata.size as i64)
            .bind(&new_metadata.user_id)
            .bind(&new_metadata.description)
            .bind(&new_metadata.file_name)
            .bind(&new_metadata.server_id)
            .bind(new_metadata.uploaded_at)
            .bind(new_metadata.download_count as i64)
            .bind(new_metadata.last_access)
            .bind(new_metadata.delete_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(created.into())
    }

    async fn get_metadata(&self, file_id: &str) -> Result<Metadata, ApplicationError> {
        let query = "SELECT * FROM application.metadata WHERE file_id = $1";

        let fetched: MetadataDTO = query_as::<_, MetadataDTO>(query)
            .bind(file_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(fetched.into())
    }

    async fn update_metadata(&self, metadata: MetadataDTO) -> Result<Metadata, ApplicationError> {
        let mut metadata = metadata;
        metadata.sanitize();

        if metadata.mime_type.is_none()
            && metadata.size.is_none()
            && metadata.user_id.is_none()
            && metadata.description.is_none()
            && metadata.file_name.is_none()
            && metadata.server_id.is_none()
            && metadata.uploaded_at.is_none()
            && metadata.download_count.is_none()
            && metadata.last_access.is_none()
            && metadata.delete_at.is_none()
        {
            return self.get_metadata(&metadata.file_id).await;
        }

        let mut builder = QueryBuilder::new("UPDATE application.metadata SET ");
        let mut separated = builder.separated(", ");

        if let Some(mime_type) = &metadata.mime_type {
            separated.push("mime_type = ");
            separated.push_bind_unseparated(mime_type);
        }
        if let Some(size) = metadata.size {
            separated.push("size = ");
            separated.push_bind_unseparated(size as i64);
        }
        if metadata.user_id.is_some() {
            separated.push("user_id = ");
            separated.push_bind_unseparated(&metadata.user_id);
        }
        if metadata.description.is_some() {
            separated.push("description = ");
            separated.push_bind_unseparated(&metadata.description);
        }
        if let Some(file_name) = &metadata.file_name {
            separated.push("file_name = ");
            separated.push_bind_unseparated(file_name);
        }
        if let Some(server_id) = &metadata.server_id {
            separated.push("server_id = ");
            separated.push_bind_unseparated(server_id);
        }
        if let Some(uploaded_at) = &metadata.uploaded_at {
            separated.push("uploaded_at = ");
            separated.push_bind_unseparated(uploaded_at);
        }
        if let Some(download_count) = metadata.download_count {
            separated.push("download_count = ");
            separated.push_bind_unseparated(download_count as i64);
        }
        if let Some(last_access) = &metadata.last_access {
            separated.push("last_access = ");
            separated.push_bind_unseparated(last_access);
        }
        if metadata.delete_at.is_some() {
            separated.push("delete_at = ");
            separated.push_bind_unseparated(metadata.delete_at);
        }

        builder.push(" WHERE file_id = ");
        builder.push_bind(&metadata.file_id);
        builder.push(" RETURNING *");

        let query = builder.build_query_as::<MetadataDTO>();

        let updated = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(updated.into())
    }

    async fn delete_metadata(&self, file_id: &str) -> Result<Metadata, ApplicationError> {
        let query = "DELETE FROM application.metadata WHERE file_id = $1 RETURNING *";

        let deleted: MetadataDTO = query_as::<_, MetadataDTO>(query)
            .bind(file_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(deleted.into())
    }

    async fn increment_download_count(&self, file_id: &str) -> Result<Metadata, ApplicationError> {
        let query = r#"
            UPDATE application.metadata
            SET download_count = download_count + 1,
                last_access = NOW()
            WHERE file_id = $1
            RETURNING *
        "#;

        let updated: MetadataDTO = query_as::<_, MetadataDTO>(query)
            .bind(file_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(updated.into())
    }

    async fn get_expired_files(&self) -> Result<Vec<Metadata>, ApplicationError> {
        let query = r#"
            SELECT * FROM application.metadata
            WHERE delete_at IS NOT NULL AND delete_at <= NOW()
        "#;

        let rows: Vec<MetadataDTO> = query_as::<_, MetadataDTO>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|dto| dto.into()).collect())
    }

    async fn get_file_ids_by_user(&self, user_id: &str) -> Result<Vec<String>, ApplicationError> {
        let query =
            "SELECT file_id FROM application.metadata WHERE user_id = $1 ORDER BY uploaded_at DESC";

        let rows: Vec<(String,)> = sqlx::query_as(query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|(id,)| id).collect())
    }
}
