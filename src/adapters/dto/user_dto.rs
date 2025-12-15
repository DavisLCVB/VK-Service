use sqlx::{postgres::PgRow, FromRow, Row};

use crate::application::dto::user_dto::UserDTO;

impl FromRow<'_, PgRow> for UserDTO {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let file_count: i64 = row.try_get("file_count")?;
        let total_space: i64 = row.try_get("total_space")?;
        let used_space: i64 = row.try_get("used_space")?;
        Ok(UserDTO {
            uid: row.try_get("uid")?,
            file_count: Some(file_count as u64),
            total_space: Some(total_space as u64),
            used_space: Some(used_space as u64),
        })
    }
}

impl UserDTO {
    pub fn sanitize(&mut self) {
        if let Some(file_count) = self.file_count {
            self.file_count = Some(std::cmp::min(file_count, i64::MAX as u64));
        }
        if let Some(total_space) = self.total_space {
            self.total_space = Some(std::cmp::min(total_space, i64::MAX as u64));
        }
        if let Some(used_space) = self.used_space {
            self.used_space = Some(std::cmp::min(used_space, i64::MAX as u64));
        }
    }
}
