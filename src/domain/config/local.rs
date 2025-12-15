use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Provider {
    #[serde(rename = "gdrive")]
    GDrive,
    #[serde(rename = "supabase")]
    Supabase,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalConfig {
    pub provider: Provider,
    #[serde(rename = "serverName")]
    pub server_name: String,
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
}
