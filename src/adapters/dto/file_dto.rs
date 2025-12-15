use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::models::metadata::Metadata;

#[derive(Debug, Serialize)]
pub struct UploadFileResponse {
    #[serde(rename = "fileId")]
    pub file_id: String,
    pub size: u64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub filename: String,
    #[serde(rename = "uploadedAt")]
    pub uploaded_at: DateTime<Utc>,
    #[serde(rename = "deleteAt")]
    pub delete_at: Option<DateTime<Utc>>,
}

impl From<Metadata> for UploadFileResponse {
    fn from(metadata: Metadata) -> Self {
        Self {
            file_id: metadata.file_id,
            size: metadata.size,
            mime_type: metadata.mime_type,
            filename: metadata.file_name,
            uploaded_at: metadata.uploaded_at,
            delete_at: metadata.delete_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateFileRequest {
    pub description: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "deleteAt")]
    pub delete_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct FileResponse {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub size: u64,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
    #[serde(rename = "uploadedAt")]
    pub uploaded_at: DateTime<Utc>,
    #[serde(rename = "downloadCount")]
    pub download_count: u64,
    #[serde(rename = "lastAccess")]
    pub last_access: DateTime<Utc>,
    #[serde(rename = "deleteAt")]
    pub delete_at: Option<DateTime<Utc>>,
}

impl From<Metadata> for FileResponse {
    fn from(metadata: Metadata) -> Self {
        Self {
            file_id: metadata.file_id,
            mime_type: metadata.mime_type,
            size: metadata.size,
            user_id: metadata.user_id,
            description: metadata.description,
            file_name: metadata.file_name,
            server_id: metadata.server_id,
            uploaded_at: metadata.uploaded_at,
            download_count: metadata.download_count,
            last_access: metadata.last_access,
            delete_at: metadata.delete_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CleanupResponse {
    #[serde(rename = "deletedCount")]
    pub deleted_count: usize,
    pub errors: Vec<String>,
}
