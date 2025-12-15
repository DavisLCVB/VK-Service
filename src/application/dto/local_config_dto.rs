use serde::{Deserialize, Serialize};

use crate::domain::config::local::{LocalConfig, Provider};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LocalConfigDTO {
    pub provider: Option<Provider>,
    #[serde(rename = "serverName")]
    pub server_name: Option<String>,
    #[serde(rename = "serverUrl")]
    pub server_url: Option<String>,
}

impl From<LocalConfig> for LocalConfigDTO {
    fn from(value: LocalConfig) -> Self {
        LocalConfigDTO {
            provider: Some(value.provider),
            server_name: Some(value.server_name),
            server_url: Some(value.server_url),
        }
    }
}

impl From<LocalConfigDTO> for LocalConfig {
    fn from(value: LocalConfigDTO) -> Self {
        LocalConfig {
            provider: value.provider.unwrap_or(Provider::GDrive),
            server_name: value.server_name.unwrap_or_default(),
            server_url: value.server_url.unwrap_or_default(),
            server_id: String::new(),
        }
    }
}
