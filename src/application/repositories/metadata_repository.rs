use async_trait::async_trait;

use crate::{
    application::{dto::metadata_dto::MetadataDTO, error::ApplicationError},
    domain::models::metadata::Metadata,
};

#[async_trait]
pub trait MetadataRepository: Send + Sync {
    async fn create_metadata(&self, metadata: MetadataDTO) -> Result<Metadata, ApplicationError>;
    async fn get_metadata(&self, file_id: &str) -> Result<Metadata, ApplicationError>;
    async fn update_metadata(&self, metadata: MetadataDTO) -> Result<Metadata, ApplicationError>;
    async fn delete_metadata(&self, file_id: &str) -> Result<Metadata, ApplicationError>;
    async fn increment_download_count(&self, file_id: &str) -> Result<Metadata, ApplicationError>;
    async fn get_expired_files(&self) -> Result<Vec<Metadata>, ApplicationError>;
    async fn get_file_ids_by_user(&self, user_id: &str) -> Result<Vec<String>, ApplicationError>;
}
