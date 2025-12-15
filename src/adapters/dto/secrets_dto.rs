use sqlx::{postgres::PgRow, FromRow, Row};

use crate::{
    application::dto::secrets_dto::SecretsDTO,
    domain::config::secrets::{GDriveSecrets, SupabaseSecrets},
};

impl FromRow<'_, PgRow> for SecretsDTO {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        // JSONB deserialization for both provider secrets
        let gdrive_secrets: Option<GDriveSecrets> =
            match row.try_get::<Option<sqlx::types::JsonValue>, _>("gdrive_secrets")? {
                Some(json) => Some(
                    serde_json::from_value(json).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                ),
                None => None,
            };

        let supabase_secrets: Option<SupabaseSecrets> =
            match row.try_get::<Option<sqlx::types::JsonValue>, _>("supabase_secrets")? {
                Some(json) => Some(
                    serde_json::from_value(json).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                ),
                None => None,
            };

        Ok(SecretsDTO {
            db_password: Some(row.try_get("db_password")?),
            db_username: Some(row.try_get("db_username")?),
            vk_secret: Some(row.try_get("vk_secret")?),
            gdrive_secrets,
            supabase_secrets,
        })
    }
}
