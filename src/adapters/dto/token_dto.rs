use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: u64,
}

#[derive(Debug, Deserialize, Default)]
pub struct GenerateTokenRequest {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
}
