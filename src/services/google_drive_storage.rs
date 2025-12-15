use async_trait::async_trait;
use reqwest::{multipart, Client};
use serde::Deserialize;

use crate::{
    application::{error::ApplicationError, services::StorageService},
    domain::{
        config::secrets::GDriveSecrets,
        models::file::{FileData, FileMetadata},
    },
    services::error::StorageError,
};

const GOOGLE_DRIVE_API_BASE: &str = "https://www.googleapis.com/drive/v3";
const GOOGLE_UPLOAD_API_BASE: &str = "https://www.googleapis.com/upload/drive/v3";

#[derive(Debug, Deserialize)]
struct ServiceAccountCredentials {
    client_email: String,
    private_key: String,
    token_uri: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct DriveFileMetadata {
    id: String,
    name: Option<String>,
    #[serde(rename = "mimeType")]
    mime_type: String,
    size: Option<String>,
}

pub struct GDriveStorageService {
    client: Client,
    folder_id: String,
    credentials: ServiceAccountCredentials,
    access_token: tokio::sync::Mutex<Option<String>>,
}

impl GDriveStorageService {
    pub fn new(secrets: GDriveSecrets) -> Result<Self, StorageError> {
        let credentials: ServiceAccountCredentials =
            serde_json::from_str(&secrets.google_credentials)
                .map_err(|e| StorageError::InvalidCredentials(e.to_string()))?;

        Ok(Self {
            client: Client::new(),
            folder_id: secrets.folder_id,
            credentials,
            access_token: tokio::sync::Mutex::new(None),
        })
    }

    async fn get_access_token(&self) -> Result<String, StorageError> {
        let token = self.access_token.lock().await;
        if let Some(ref t) = *token {
            return Ok(t.clone());
        }
        drop(token);

        let jwt = self.create_jwt()?;

        let response = self
            .client
            .post(&self.credentials.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await?;

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| StorageError::Unauthorized(e.to_string()))?;

        let mut token = self.access_token.lock().await;
        *token = Some(token_response.access_token.clone());

        Ok(token_response.access_token)
    }

    fn create_jwt(&self) -> Result<String, StorageError> {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
        use serde::Serialize;
        use std::time::{SystemTime, UNIX_EPOCH};

        #[derive(Serialize)]
        struct Claims {
            iss: String,
            scope: String,
            aud: String,
            exp: u64,
            iat: u64,
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let claims = Claims {
            iss: self.credentials.client_email.clone(),
            scope: "https://www.googleapis.com/auth/drive.file".to_string(),
            aud: self.credentials.token_uri.clone(),
            exp: now + 3600,
            iat: now,
        };

        let key = EncodingKey::from_rsa_pem(self.credentials.private_key.as_bytes())
            .map_err(|e| StorageError::InvalidCredentials(e.to_string()))?;

        encode(&Header::new(Algorithm::RS256), &claims, &key)
            .map_err(|e| StorageError::InternalError(e.to_string()))
    }
}

#[async_trait]
impl StorageService for GDriveStorageService {
    async fn upload(&self, file_data: FileData) -> Result<FileMetadata, ApplicationError> {
        let token = self.get_access_token().await?;

        let file_metadata = serde_json::json!({
            "name": file_data.filename,
            "mimeType": file_data.mime_type,
            "parents": [self.folder_id],
        });

        let metadata_part = multipart::Part::text(file_metadata.to_string())
            .mime_str("application/json")
            .map_err(|e| StorageError::InternalError(e.to_string()))?;

        let file_part = multipart::Part::bytes(file_data.content.clone())
            .mime_str(&file_data.mime_type)
            .map_err(|e| StorageError::InternalError(e.to_string()))?;

        let form = multipart::Form::new()
            .part("metadata", metadata_part)
            .part("file", file_part);

        let url = format!(
            "{}/files?uploadType=multipart&fields=id,name,mimeType,size",
            GOOGLE_UPLOAD_API_BASE
        );

        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
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

        let drive_metadata: DriveFileMetadata = response
            .json()
            .await
            .map_err(|e| StorageError::InternalError(e.to_string()))?;

        Ok(FileMetadata {
            file_id: drive_metadata.id,
            size: file_data.size(),
            mime_type: drive_metadata.mime_type,
            filename: drive_metadata.name,
            provider: "gdrive".to_string(),
        })
    }

    async fn download(&self, file_id: &str) -> Result<Vec<u8>, ApplicationError> {
        let token = self.get_access_token().await?;

        let url = format!("{}/files/{}?alt=media", GOOGLE_DRIVE_API_BASE, file_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(token)
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
        let token = self.get_access_token().await?;

        let url = format!("{}/files/{}", GOOGLE_DRIVE_API_BASE, file_id);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(token)
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
        let token = self.get_access_token().await?;

        let url = format!(
            "{}/files/{}?fields=id,name,mimeType,size",
            GOOGLE_DRIVE_API_BASE, file_id
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(token)
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

        let drive_metadata: DriveFileMetadata = response
            .json()
            .await
            .map_err(|e| StorageError::InternalError(e.to_string()))?;

        let size = drive_metadata
            .size
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(FileMetadata {
            file_id: drive_metadata.id,
            size,
            mime_type: drive_metadata.mime_type,
            filename: drive_metadata.name,
            provider: "gdrive".to_string(),
        })
    }
}
