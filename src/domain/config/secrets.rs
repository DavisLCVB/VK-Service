use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GDriveSecrets {
    #[serde(rename = "folderId")]
    pub folder_id: String,
    #[serde(rename = "googleCredentials")]
    pub google_credentials: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupabaseSecrets {
    #[serde(rename = "endpoint")]
    pub endpoint: String,
    #[serde(rename = "region")]
    pub region: String,
    #[serde(rename = "accessKeyId")]
    pub access_key_id: String,
    #[serde(rename = "secretAccessKey")]
    pub secret_access_key: String,
    #[serde(rename = "bucketName")]
    pub bucket_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Secrets {
    #[serde(rename = "dbPassword")]
    pub db_password: String,
    #[serde(rename = "dbUsername")]
    pub db_username: String,
    #[serde(rename = "dbName")]
    pub vk_secret: String,
    #[serde(rename = "gdriveSecrets")]
    pub gdrive_secrets: Option<GDriveSecrets>,
    #[serde(rename = "supabaseSecrets")]
    pub supabase_secrets: Option<SupabaseSecrets>,
}
