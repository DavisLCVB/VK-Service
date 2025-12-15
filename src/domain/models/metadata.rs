use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub file_id: String,
    pub mime_type: String,
    pub size: u64,
    pub user_id: Option<String>,
    pub description: Option<String>,
    pub file_name: String,
    pub server_id: String,
    pub uploaded_at: DateTime<Utc>,
    pub download_count: u64,
    pub last_access: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_at: Option<DateTime<Utc>>,
}
