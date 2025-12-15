use async_trait::async_trait;
use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};

use crate::{
    application::{error::ApplicationError, services::StorageService},
    domain::{
        config::secrets::SupabaseSecrets,
        models::file::{FileData, FileMetadata},
    },
    services::error::StorageError,
};

pub struct SupabaseStorageService {
    client: Client,
    bucket_name: String,
}

impl SupabaseStorageService {
    pub async fn new(secrets: SupabaseSecrets) -> Result<Self, StorageError> {
        let credentials = Credentials::new(
            &secrets.access_key_id,
            &secrets.secret_access_key,
            None,
            None,
            "supabase-storage",
        );

        // Build S3 config directly without loading from environment
        // This avoids network calls to AWS metadata service
        let config = aws_sdk_s3::config::Builder::new()
            .credentials_provider(credentials)
            .region(Region::new(secrets.region))
            .endpoint_url(&secrets.endpoint)
            .force_path_style(true) // Required for S3-compatible services like Supabase
            .build();

        let client = Client::from_conf(config);

        Ok(Self {
            client,
            bucket_name: secrets.bucket_name,
        })
    }

    fn generate_file_path(&self, filename: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let safe_filename = filename
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        format!("{}/{}", timestamp, safe_filename)
    }
}

#[async_trait]
impl StorageService for SupabaseStorageService {
    async fn upload(&self, file_data: FileData) -> Result<FileMetadata, ApplicationError> {
        let file_path = self.generate_file_path(&file_data.filename);

        let byte_stream = ByteStream::from(file_data.content.clone());

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&file_path)
            .body(byte_stream)
            .content_type(&file_data.mime_type)
            .send()
            .await
            .map_err(|e| {
                StorageError::ProviderError(format!("S3 upload failed: {}", e))
            })?;

        Ok(FileMetadata {
            file_id: file_path,
            size: file_data.size(),
            mime_type: file_data.mime_type.clone(),
            filename: Some(file_data.filename),
            provider: "supabase".to_string(),
        })
    }

    async fn download(&self, file_id: &str) -> Result<Vec<u8>, ApplicationError> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(file_id)
            .send()
            .await
            .map_err(|e| {
                let error_str = e.to_string();
                if error_str.contains("NoSuchKey") || error_str.contains("404") {
                    StorageError::NotFound(file_id.to_string())
                } else {
                    StorageError::ProviderError(format!("S3 download failed: {}", e))
                }
            })?;

        let bytes = response
            .body
            .collect()
            .await
            .map_err(|e| StorageError::NetworkError(e.to_string()))?
            .into_bytes();

        Ok(bytes.to_vec())
    }

    async fn delete(&self, file_id: &str) -> Result<(), ApplicationError> {
        // First check if the object exists
        let _head = self
            .client
            .head_object()
            .bucket(&self.bucket_name)
            .key(file_id)
            .send()
            .await
            .map_err(|e| {
                let error_str = e.to_string();
                if error_str.contains("NotFound") || error_str.contains("404") {
                    StorageError::NotFound(file_id.to_string())
                } else {
                    StorageError::ProviderError(format!("S3 head object failed: {}", e))
                }
            })?;

        // If exists, delete it
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(file_id)
            .send()
            .await
            .map_err(|e| {
                StorageError::ProviderError(format!("S3 delete failed: {}", e))
            })?;

        Ok(())
    }

    async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata, ApplicationError> {
        let response = self
            .client
            .head_object()
            .bucket(&self.bucket_name)
            .key(file_id)
            .send()
            .await
            .map_err(|e| {
                let error_str = e.to_string();
                if error_str.contains("NotFound") || error_str.contains("404") {
                    StorageError::NotFound(file_id.to_string())
                } else {
                    StorageError::ProviderError(format!("S3 head object failed: {}", e))
                }
            })?;

        let size = response.content_length().unwrap_or(0) as u64;
        let mime_type = response
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let filename = file_id.split('/').last().map(|s| s.to_string());

        Ok(FileMetadata {
            file_id: file_id.to_string(),
            size,
            mime_type,
            filename,
            provider: "supabase".to_string(),
        })
    }
}
