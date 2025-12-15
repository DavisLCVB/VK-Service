use sqlx::{postgres::PgRow, FromRow, Row};

use crate::{application::dto::local_config_dto::LocalConfigDTO, domain::config::local::Provider};

impl FromRow<'_, PgRow> for LocalConfigDTO {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        // Provider enum parsing from TEXT column
        let provider_str: String = row.try_get("provider")?;
        let provider = match provider_str.as_str() {
            "gdrive" => Provider::GDrive,
            "supabase" => Provider::Supabase,
            _ => {
                return Err(sqlx::Error::Decode(
                    format!("Unknown provider: {}", provider_str).into(),
                ))
            }
        };

        Ok(LocalConfigDTO {
            provider: Some(provider),
            server_name: Some(row.try_get("server_name")?),
            server_url: Some(row.try_get("server_url")?),
        })
    }
}

impl LocalConfigDTO {
    pub fn sanitize(&mut self) {}
}
