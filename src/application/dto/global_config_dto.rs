use serde::{Deserialize, Serialize};

use crate::domain::config::global::GlobalConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfigDTO {
    #[serde(rename = "mimeTypes")]
    pub mime_types: Option<Vec<String>>,
    #[serde(rename = "maxSize")]
    pub max_size: Option<u64>,
    #[serde(rename = "chunkSize")]
    pub chunk_size: Option<u64>,
    #[serde(rename = "tempFileLife")]
    pub temp_file_life: Option<u64>,
    #[serde(rename = "defaultQuota")]
    pub default_quota: Option<u64>,
}

impl GlobalConfigDTO {
    pub fn sanitize(&mut self) {
        if let Some(ref mut mime_types) = self.mime_types {
            mime_types.retain(|s| !s.trim().is_empty());
        }
        if let Some(max_size) = self.max_size {
            self.max_size = Some(std::cmp::min(max_size, i64::MAX as u64));
        }
        if let Some(chunk_size) = self.chunk_size {
            self.chunk_size = Some(std::cmp::min(chunk_size, i64::MAX as u64));
        }
        if let Some(temp_file_life) = self.temp_file_life {
            self.temp_file_life = Some(std::cmp::min(temp_file_life, i64::MAX as u64));
        }
        if let Some(default_quota) = self.default_quota {
            self.default_quota = Some(std::cmp::min(default_quota, i64::MAX as u64));
        }
    }
}

impl From<GlobalConfig> for GlobalConfigDTO {
    fn from(value: GlobalConfig) -> Self {
        GlobalConfigDTO {
            mime_types: Some(value.mime_types),
            max_size: Some(value.max_size),
            chunk_size: Some(value.chunk_size),
            temp_file_life: Some(value.temp_file_life),
            default_quota: Some(value.default_quota),
        }
    }
}

impl From<GlobalConfigDTO> for GlobalConfig {
    fn from(value: GlobalConfigDTO) -> Self {
        GlobalConfig {
            mime_types: value.mime_types.unwrap_or_default(),
            max_size: value.max_size.unwrap_or(0),
            chunk_size: value.chunk_size.unwrap_or(0),
            temp_file_life: value.temp_file_life.unwrap_or(0),
            default_quota: value.default_quota.unwrap_or(0),
        }
    }
}
