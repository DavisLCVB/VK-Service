use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    #[serde(rename = "mimeTypes")]
    pub mime_types: Vec<String>,
    #[serde(rename = "maxSize")]
    pub max_size: u64,
    #[serde(rename = "chunkSize")]
    pub chunk_size: u64,
    #[serde(rename = "tempFileLife")]
    pub temp_file_life: u64,
    #[serde(rename = "defaultQuota")]
    pub default_quota: u64,
}
