use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct FileData {
    pub content: Vec<u8>,
    pub filename: String,
    pub mime_type: String,
}

impl FileData {
    pub fn new(content: Vec<u8>, filename: String, mime_type: String) -> Self {
        Self {
            content,
            filename,
            mime_type,
        }
    }

    pub fn validate_size(&self, max_size: u64) -> bool {
        (self.content.len() as u64) <= max_size
    }

    pub fn size(&self) -> u64 {
        self.content.len() as u64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: String,
    pub size: u64,
    pub mime_type: String,
    pub filename: Option<String>,
    pub provider: String,
}
