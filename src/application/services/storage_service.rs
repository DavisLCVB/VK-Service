use async_trait::async_trait;

use crate::{
    application::error::ApplicationError,
    domain::models::file::{FileData, FileMetadata},
};

#[async_trait]
pub trait StorageService: Send + Sync {
    async fn upload(&self, file_data: FileData) -> Result<FileMetadata, ApplicationError>;
    async fn download(&self, file_id: &str) -> Result<Vec<u8>, ApplicationError>;
    async fn delete(&self, file_id: &str) -> Result<(), ApplicationError>;
    async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata, ApplicationError>;
}
