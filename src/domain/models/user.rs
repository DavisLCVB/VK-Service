use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct User {
    pub uid: Uuid,
    #[serde(rename = "fileCount")]
    pub file_count: u64,
    #[serde(rename = "totalSpace")]
    pub total_space: u64,
    #[serde(rename = "usedSpace")]
    pub used_space: u64,
}
