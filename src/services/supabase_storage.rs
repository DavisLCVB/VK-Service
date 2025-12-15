use async_trait::async_trait;
use reqwest::{multipart, Client};

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
    storage_url: String,
    api_key: String,
    bucket_name: String,
}

impl SupabaseStorageService {
    pub fn new(secrets: SupabaseSecrets) -> Self {
        Self {
            client: Client::new(),
            storage_url: secrets.storage_url.trim_end_matches('/').to_string(),
            api_key: secrets.api_key,
            bucket_name: secrets.bucket_name,
        }
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

        let file_part = multipart::Part::bytes(file_data.content.clone())
            .file_name(file_data.filename.clone())
            .mime_str(&file_data.mime_type)
            .map_err(|e| StorageError::InternalError(e.to_string()))?;

        let form = multipart::Form::new().part("file", file_part);

        let url = format!(
            "{}/object/{}/{}",
            self.storage_url, self.bucket_name, file_path
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("apikey", &self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(StorageError::from)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(
                StorageError::ProviderError(format!("Upload failed: {}", error_text)).into(),
            );
        }

        Ok(FileMetadata {
            file_id: file_path,
            size: file_data.size(),
            mime_type: file_data.mime_type.clone(),
            filename: Some(file_data.filename),
            provider: "supabase".to_string(),
        })
    }

    async fn download(&self, file_id: &str) -> Result<Vec<u8>, ApplicationError> {
        let url = format!(
            "{}/object/{}/{}",
            self.storage_url, self.bucket_name, file_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("apikey", &self.api_key)
            .send()
            .await
            .map_err(StorageError::from)?;

        if response.status().as_u16() == 404 {
            return Err(StorageError::NotFound(file_id.to_string()).into());
        }

        if !response.status().is_success() {
            return Err(StorageError::ProviderError(format!(
                "Download failed with status: {}",
                response.status()
            ))
            .into());
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| StorageError::NetworkError(e.to_string()))?;

        Ok(bytes.to_vec())
    }

    async fn delete(&self, file_id: &str) -> Result<(), ApplicationError> {
        let url = format!(
            "{}/object/{}/{}",
            self.storage_url, self.bucket_name, file_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("apikey", &self.api_key)
            .send()
            .await
            .map_err(StorageError::from)?;

        if response.status().as_u16() == 404 {
            return Err(StorageError::NotFound(file_id.to_string()).into());
        }

        if !response.status().is_success() {
            return Err(StorageError::ProviderError(format!(
                "Delete failed with status: {}",
                response.status()
            ))
            .into());
        }

        Ok(())
    }

    async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata, ApplicationError> {
        let url = format!(
            "{}/object/{}/{}",
            self.storage_url, self.bucket_name, file_id
        );

        let response = self
            .client
            .head(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("apikey", &self.api_key)
            .send()
            .await
            .map_err(StorageError::from)?;

        if response.status().as_u16() == 404 {
            return Err(StorageError::NotFound(file_id.to_string()).into());
        }

        if !response.status().is_success() {
            return Err(StorageError::ProviderError(format!(
                "Get metadata failed with status: {}",
                response.status()
            ))
            .into());
        }

        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let filename = file_id.split('/').last().map(|s| s.to_string());

        Ok(FileMetadata {
            file_id: file_id.to_string(),
            size: content_length,
            mime_type: content_type,
            filename,
            provider: "supabase".to_string(),
        })
    }
}
