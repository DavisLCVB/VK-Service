use thiserror::Error;

use crate::application::error::ApplicationError;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    #[error("Storage provider error: {0}")]
    ProviderError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<StorageError> for ApplicationError {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::NotFound(_) => ApplicationError::NotFound,
            StorageError::Unauthorized(msg)
            | StorageError::NetworkError(msg)
            | StorageError::InvalidCredentials(msg)
            | StorageError::ProviderError(msg)
            | StorageError::InternalError(msg) => {
                ApplicationError::InternalError(format!("Storage error: {}", msg))
            }
        }
    }
}

impl From<reqwest::Error> for StorageError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            StorageError::NetworkError("Request timeout".to_string())
        } else if error.is_connect() {
            StorageError::NetworkError(format!("Connection failed: {}", error))
        } else if let Some(status) = error.status() {
            match status.as_u16() {
                404 => StorageError::NotFound(error.to_string()),
                401 | 403 => StorageError::Unauthorized(error.to_string()),
                _ => StorageError::ProviderError(error.to_string()),
            }
        } else {
            StorageError::InternalError(error.to_string())
        }
    }
}
