use sqlx::{postgres::PgRow, FromRow, Row};

use crate::application::dto::metadata_dto::MetadataDTO;

impl FromRow<'_, PgRow> for MetadataDTO {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let size: i64 = row.try_get("size")?;
        let download_count: i64 = row.try_get("download_count")?;

        Ok(MetadataDTO {
            file_id: row.try_get("file_id")?,
            mime_type: Some(row.try_get("mime_type")?),
            size: Some(size as u64),
            user_id: row.try_get("user_id")?,
            description: row.try_get("description")?,
            file_name: Some(row.try_get("file_name")?),
            server_id: Some(row.try_get("server_id")?),
            uploaded_at: Some(row.try_get("uploaded_at")?),
            download_count: Some(download_count as u64),
            last_access: Some(row.try_get("last_access")?),
            delete_at: row.try_get("delete_at")?,
        })
    }
}

impl MetadataDTO {
    pub fn sanitize(&mut self) {
        if let Some(size) = self.size {
            self.size = Some(std::cmp::min(size, i64::MAX as u64));
        }
        if let Some(download_count) = self.download_count {
            self.download_count = Some(std::cmp::min(download_count, i64::MAX as u64));
        }
    }
}
