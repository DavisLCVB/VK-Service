use serde::{Deserialize, Serialize};

use crate::domain::config::secrets::{GDriveSecrets, Secrets, SupabaseSecrets};

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretsDTO {
    #[serde(rename = "dbPassword")]
    pub db_password: Option<String>,
    #[serde(rename = "dbUsername")]
    pub db_username: Option<String>,
    #[serde(rename = "dbName")]
    pub vk_secret: Option<String>,
    #[serde(rename = "gdriveSecrets")]
    pub gdrive_secrets: Option<GDriveSecrets>,
    #[serde(rename = "supabaseSecrets")]
    pub supabase_secrets: Option<SupabaseSecrets>,
}

impl SecretsDTO {
    pub fn sanitize(&mut self) {
        if let Some(ref mut db_password) = self.db_password {
            *db_password = db_password.trim().to_string();
        }
        if let Some(ref mut db_username) = self.db_username {
            *db_username = db_username.trim().to_string();
        }
        if let Some(ref mut vk_secret) = self.vk_secret {
            *vk_secret = vk_secret.trim().to_string();
        }
    }
}

impl From<Secrets> for SecretsDTO {
    fn from(value: Secrets) -> Self {
        SecretsDTO {
            db_password: Some(value.db_password),
            db_username: Some(value.db_username),
            vk_secret: Some(value.vk_secret),
            gdrive_secrets: value.gdrive_secrets,
            supabase_secrets: value.supabase_secrets,
        }
    }
}

impl From<SecretsDTO> for Secrets {
    fn from(value: SecretsDTO) -> Self {
        Secrets {
            db_password: value.db_password.unwrap_or_default(),
            db_username: value.db_username.unwrap_or_default(),
            vk_secret: value.vk_secret.unwrap_or_default(),
            gdrive_secrets: value.gdrive_secrets,
            supabase_secrets: value.supabase_secrets,
        }
    }
}
