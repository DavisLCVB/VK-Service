use async_trait::async_trait;
use sqlx::{query_as, QueryBuilder};

use crate::{
    application::{
        dto::user_dto::UserDTO, error::ApplicationError,
        repositories::user_repository::UserRepository,
    },
    domain::models::user::User,
};

pub struct PgUserRepository {
    pool: sqlx::PgPool,
}

impl PgUserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create_user(&self, user: UserDTO, new_space: u64) -> Result<User, ApplicationError> {
        let query = r#"
            INSERT INTO application.users (uid, file_count, total_space, used_space) 
            VALUES ($1, $2, $3, $4) 
            RETURNING *
        "#;
        let new_user = User {
            uid: user.uid,
            file_count: 0,
            total_space: new_space,
            used_space: 0,
        };
        let created_user: UserDTO = query_as::<_, UserDTO>(&query)
            .bind(&new_user.uid)
            .bind(new_user.file_count as i64)
            .bind(new_user.total_space as i64)
            .bind(new_user.used_space as i64)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;
        Ok(created_user.into())
    }

    async fn get_user(&self, user: UserDTO) -> Result<User, ApplicationError> {
        let query = "SELECT * FROM application.users WHERE uid = $1";
        let fetched_user: UserDTO = query_as::<_, UserDTO>(query)
            .bind(&user.uid)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;
        Ok(fetched_user.into())
    }

    async fn update_user(&self, user: UserDTO) -> Result<User, ApplicationError> {
        let mut user = user;
        user.sanitize();
        if user.file_count.is_none() && user.total_space.is_none() && user.used_space.is_none() {
            return self.get_user(user).await;
        }
        let mut builder = QueryBuilder::new("UPDATE application.users SET ");
        let mut separated = builder.separated(", ");
        if let Some(file_count) = user.file_count {
            separated.push("file_count = ");
            separated.push_bind_unseparated(file_count as i64);
        }
        if let Some(total_space) = user.total_space {
            separated.push("total_space = ");
            separated.push_bind_unseparated(total_space as i64);
        }
        if let Some(used_space) = user.used_space {
            separated.push("used_space = ");
            separated.push_bind_unseparated(used_space as i64);
        }
        builder.push(" WHERE uid = ");
        builder.push_bind(&user.uid);
        builder.push(" RETURNING *");
        let query = builder.build_query_as::<UserDTO>();
        let updated_user = query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;
        Ok(updated_user.into())
    }

    async fn delete_user(&self, user: UserDTO) -> Result<User, ApplicationError> {
        let query = "DELETE FROM application.users WHERE uid = $1 RETURNING *";
        let deleted_user: UserDTO = query_as::<_, UserDTO>(query)
            .bind(&user.uid)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApplicationError::DatabaseError(e.to_string()))?;
        Ok(deleted_user.into())
    }
}
