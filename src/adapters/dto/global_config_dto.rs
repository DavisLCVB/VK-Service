use sqlx::{postgres::PgRow, FromRow, Row};

use crate::application::dto::global_config_dto::GlobalConfigDTO;

impl FromRow<'_, PgRow> for GlobalConfigDTO {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let mime_types: Vec<String> = row.try_get("mime_types")?;
        let max_size: i64 = row.try_get("max_size")?;
        let chunk_size: i64 = row.try_get("chunk_size")?;
        let temp_file_life: i64 = row.try_get("temp_file_life")?;
        let default_quota: i64 = row.try_get("default_quota")?;

        Ok(GlobalConfigDTO {
            mime_types: Some(mime_types),
            max_size: Some(max_size as u64),
            chunk_size: Some(chunk_size as u64),
            temp_file_life: Some(temp_file_life as u64),
            default_quota: Some(default_quota as u64),
        })
    }
}
