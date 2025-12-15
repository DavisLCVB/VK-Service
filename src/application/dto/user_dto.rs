use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDTO {
    pub uid: Uuid,
    #[serde(rename = "fileCount")]
    pub file_count: Option<u64>,
    #[serde(rename = "totalSpace")]
    pub total_space: Option<u64>,
    #[serde(rename = "usedSpace")]
    pub used_space: Option<u64>,
}

impl UserDTO {
    /// Crea un UserDTO vacío para consultas (solo con uid)
    pub fn for_query(uid: Uuid) -> Self {
        Self {
            uid,
            file_count: None,
            total_space: None,
            used_space: None,
        }
    }

    /// Crea un UserDTO para actualización parcial con campos específicos
    pub fn for_update(uid: Uuid) -> Self {
        Self {
            uid,
            file_count: None,
            total_space: None,
            used_space: None,
        }
    }
}

impl From<User> for UserDTO {
    fn from(value: User) -> Self {
        UserDTO {
            uid: value.uid,
            file_count: Some(value.file_count),
            total_space: Some(value.total_space),
            used_space: Some(value.used_space),
        }
    }
}

impl From<UserDTO> for User {
    fn from(value: UserDTO) -> Self {
        User {
            uid: value.uid,
            file_count: value.file_count.unwrap_or(0),
            total_space: value.total_space.unwrap_or(0),
            used_space: value.used_space.unwrap_or(0),
        }
    }
}
