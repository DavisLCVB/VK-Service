use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::{
    application::{
        dto::user_dto::UserDTO,
        error::ApplicationError,
        repositories::{metadata_repository::MetadataRepository, user_repository::UserRepository},
    },
    domain::{config::global::GlobalConfig, models::user::User},
};

pub struct UserController;

#[derive(Deserialize)]
pub struct CreateUser {
    uid: Uuid,
}

impl UserController {
    pub async fn create_user(
        State(global_config): State<Arc<Mutex<GlobalConfig>>>,
        State(user_repo): State<Arc<dyn UserRepository>>,
        Json(body): Json<CreateUser>,
    ) -> Result<Json<User>, ApplicationError> {
        let mut user = User::default();
        user.uid = body.uid;
        let user_dto = UserDTO::from(user);
        let default_quota = {
            let gc = global_config.lock().unwrap();
            gc.default_quota
        };
        let user = user_repo.create_user(user_dto, default_quota).await?;
        Ok(Json(user))
    }

    pub async fn get_user(
        State(user_repo): State<Arc<dyn UserRepository>>,
        Path(user_id): Path<Uuid>,
    ) -> Result<Json<User>, ApplicationError> {
        let user_dto = UserDTO::for_query(user_id);
        let user = user_repo.get_user(user_dto).await?;
        Ok(Json(user))
    }

    pub async fn update_user(
        State(user_repo): State<Arc<dyn UserRepository>>,
        Path(user_id): Path<Uuid>,
        Json(body): Json<UserDTO>,
    ) -> Result<Json<User>, ApplicationError> {
        let mut user_dto = body;
        user_dto.uid = user_id;
        let user = user_repo.update_user(user_dto).await?;
        Ok(Json(user))
    }

    pub async fn delete_user(
        State(user_repo): State<Arc<dyn UserRepository>>,
        Path(user_id): Path<Uuid>,
    ) -> Result<Json<User>, ApplicationError> {
        let user_dto = UserDTO::for_query(user_id);
        let user = user_repo.delete_user(user_dto).await?;
        Ok(Json(user))
    }

    pub async fn get_user_files(
        State(metadata_repo): State<Arc<dyn MetadataRepository>>,
        Path(user_id): Path<Uuid>,
    ) -> Result<Json<Vec<String>>, ApplicationError> {
        info!("Getting file IDs for user: {}", user_id);
        let user_id_str = user_id.to_string();
        let file_ids = metadata_repo.get_file_ids_by_user(&user_id_str).await?;
        Ok(Json(file_ids))
    }
}
