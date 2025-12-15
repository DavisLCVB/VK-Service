use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::models::metadata::Metadata;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetadataDTO {
    #[serde(default)]
    pub file_id: String,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
    pub user_id: Option<String>,
    pub description: Option<String>,
    pub file_name: Option<String>,
    pub server_id: Option<String>,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub download_count: Option<u64>,
    pub last_access: Option<DateTime<Utc>>,
    pub delete_at: Option<DateTime<Utc>>,
}

impl From<Metadata> for MetadataDTO {
    fn from(value: Metadata) -> Self {
        MetadataDTO {
            file_id: value.file_id,
            mime_type: Some(value.mime_type),
            size: Some(value.size),
            user_id: value.user_id,
            description: value.description,
            file_name: Some(value.file_name),
            server_id: Some(value.server_id),
            uploaded_at: Some(value.uploaded_at),
            download_count: Some(value.download_count),
            last_access: Some(value.last_access),
            delete_at: value.delete_at,
        }
    }
}

impl From<MetadataDTO> for Metadata {
    fn from(value: MetadataDTO) -> Self {
        Metadata {
            file_id: value.file_id,
            mime_type: value.mime_type.unwrap_or_default(),
            size: value.size.unwrap_or(0),
            user_id: value.user_id,
            description: value.description,
            file_name: value.file_name.unwrap_or_default(),
            server_id: value.server_id.unwrap_or_default(),
            uploaded_at: value.uploaded_at.unwrap_or_else(Utc::now),
            download_count: value.download_count.unwrap_or(0),
            last_access: value.last_access.unwrap_or_else(Utc::now),
            delete_at: value.delete_at,
        }
    }
}
