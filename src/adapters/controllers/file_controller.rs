use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Json,
};
use chrono::{Duration, Utc};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    adapters::{
        dto::{
            file_dto::{CleanupResponse, FileResponse, UpdateFileRequest, UploadFileResponse},
            token_dto::{GenerateTokenRequest, TokenResponse},
        },
        state::AppState,
    },
    application::{
        dto::{metadata_dto::MetadataDTO, user_dto::UserDTO},
        error::ApplicationError,
    },
    domain::models::file::FileData,
};

pub struct FileController;

impl FileController {
    /// Genera un token de un solo uso para subir archivos
    /// POST /api/v1/files/token
    /// Body: {} para usuarios anónimos, {"userId": "uuid"} para usuarios específicos
    pub async fn generate_upload_token(
        State(app_state): State<AppState>,
        Json(body): Json<GenerateTokenRequest>,
    ) -> Result<(StatusCode, Json<TokenResponse>), ApplicationError> {
        info!("Generating upload token for user_id: {:?}", body.user_id);

        // Validar que el usuario existe si se proporciona user_id
        if let Some(ref user_id_str) = body.user_id {
            let uid = Uuid::parse_str(user_id_str).map_err(|e| {
                warn!("Invalid UUID provided: {}, error: {}", user_id_str, e);
                ApplicationError::BadRequest("Invalid user ID format".to_string())
            })?;

            let user_dto = UserDTO::for_query(uid);
            app_state.user_repository.get_user(user_dto).await?;
            info!("User validated successfully: {}", user_id_str);
        } else {
            info!("Generating anonymous token");
        }

        const TOKEN_TTL_SECONDS: u64 = 300; // 5 minutos

        let token = app_state
            .token_repository
            .generate_token(body.user_id.clone(), TOKEN_TTL_SECONDS)
            .await?;

        info!("Token generated successfully: {}", token);

        Ok((
            StatusCode::CREATED,
            Json(TokenResponse {
                token,
                expires_in: TOKEN_TTL_SECONDS,
            }),
        ))
    }

    pub async fn upload_file(
        State(app_state): State<AppState>,
        headers: HeaderMap,
        mut multipart: Multipart,
    ) -> Result<(StatusCode, Json<UploadFileResponse>), ApplicationError> {
        // VALIDAR TOKEN ANTES DE PARSEAR MULTIPART (fail-fast)
        let token = headers
            .get("X-Upload-Token")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApplicationError::Unauthorized)?;

        let token_user_id = app_state
            .token_repository
            .verify_and_consume_token(token)
            .await?;

        info!("Token verified, associated user_id: {:?}", token_user_id);

        let mut file_bytes: Option<Vec<u8>> = None;
        let mut filename: Option<String> = None;
        let mut mime_type: Option<String> = None;
        let mut file_type: Option<String> = None;
        let mut user_id: Option<String> = None;
        let mut description: Option<String> = None;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| {
                warn!("Invalid multipart data: {}", e);
                ApplicationError::BadRequest("Invalid request format".to_string())
            })?
        {
            let name = field.name().unwrap_or("").to_string();

            match name.as_str() {
                "file" => {
                    file_bytes = Some(
                        field
                            .bytes()
                            .await
                            .map_err(|e| {
                                warn!("Cannot read file bytes: {}", e);
                                ApplicationError::BadRequest("Invalid file data".to_string())
                            })?
                            .to_vec(),
                    );
                }
                "filename" => {
                    filename = Some(field.text().await.map_err(|e| {
                        warn!("Invalid filename field: {}", e);
                        ApplicationError::BadRequest("Invalid request data".to_string())
                    })?);
                }
                "mime_type" => {
                    mime_type = Some(field.text().await.map_err(|e| {
                        warn!("Invalid mime_type field: {}", e);
                        ApplicationError::BadRequest("Invalid request data".to_string())
                    })?);
                }
                "type" => {
                    file_type = Some(field.text().await.map_err(|e| {
                        warn!("Invalid type field: {}", e);
                        ApplicationError::BadRequest("Invalid request data".to_string())
                    })?);
                }
                "user_id" => {
                    user_id = Some(field.text().await.map_err(|e| {
                        warn!("Invalid user_id field: {}", e);
                        ApplicationError::BadRequest("Invalid request data".to_string())
                    })?);
                }
                "description" => {
                    description = Some(field.text().await.map_err(|e| {
                        warn!("Invalid description field: {}", e);
                        ApplicationError::BadRequest("Invalid request data".to_string())
                    })?);
                }
                _ => {}
            }
        }

        let file_bytes = file_bytes
            .ok_or_else(|| {
                warn!("Missing required 'file' field in upload");
                ApplicationError::BadRequest("Missing required field".to_string())
            })?;
        let filename = filename
            .ok_or_else(|| {
                warn!("Missing required 'filename' field in upload");
                ApplicationError::BadRequest("Missing required field".to_string())
            })?;
        let mime_type = mime_type
            .ok_or_else(|| {
                warn!("Missing required 'mime_type' field in upload");
                ApplicationError::BadRequest("Missing required field".to_string())
            })?;
        let file_type = file_type
            .ok_or_else(|| {
                warn!("Missing required 'type' field in upload");
                ApplicationError::BadRequest("Missing required field".to_string())
            })?;

        let (max_size, mime_types, temp_file_life) = {
            let gc = app_state.global_config.lock().unwrap();
            (gc.max_size, gc.mime_types.clone(), gc.temp_file_life)
        };

        if !mime_types.contains(&mime_type) {
            return Err(ApplicationError::BadRequest(format!(
                "MIME type '{}' not allowed",
                mime_type
            )));
        }

        let file_size = file_bytes.len() as u64;
        if file_size > max_size {
            return Err(ApplicationError::PayloadTooLarge);
        }

        if file_type != "temporal" && file_type != "permanent" {
            return Err(ApplicationError::BadRequest(
                "Invalid 'type' field: must be 'temporal' or 'permanent'".to_string(),
            ));
        }

        if file_type == "permanent" && user_id.is_none() {
            return Err(ApplicationError::BadRequest(
                "Missing 'user_id' for permanent file".to_string(),
            ));
        }

        // VALIDAR CONSISTENCIA: user_id del token vs user_id del multipart
        if let Some(ref multipart_user_id) = user_id {
            match &token_user_id {
                Some(token_uid) if token_uid != multipart_user_id => {
                    error!(
                        "Token user_id '{}' does not match multipart user_id '{}'",
                        token_uid, multipart_user_id
                    );
                    return Err(ApplicationError::Unauthorized);
                }
                None => {
                    // Token anónimo pero upload de usuario
                    error!(
                        "Anonymous token used for user-specific upload with user_id '{}'",
                        multipart_user_id
                    );
                    return Err(ApplicationError::Unauthorized);
                }
                _ => {} // Token y multipart coinciden
            }
        } else if token_user_id.is_some() {
            // Token de usuario pero upload anónimo
            return Err(ApplicationError::Unauthorized);
        }

        let user = if file_type == "permanent" {
            let uid_str = user_id.as_ref().unwrap();
            let uid = Uuid::parse_str(uid_str)
                .map_err(|_| ApplicationError::BadRequest(format!("Invalid UUID: {}", uid_str)))?;

            let user_dto = UserDTO::for_query(uid);
            let user = app_state.user_repository.get_user(user_dto).await?;

            if user.used_space + file_size > user.total_space {
                return Err(ApplicationError::InsufficientStorage);
            }

            Some(user)
        } else {
            None
        };

        let file_data = FileData::new(file_bytes, filename.clone(), mime_type.clone());
        let storage_metadata = {
            let service = app_state.storage_service.get();
            service.upload(file_data).await?
        };

        let delete_at = if file_type == "temporal" {
            Some(Utc::now() + Duration::seconds(temp_file_life as i64))
        } else {
            None
        };

        let metadata_dto = MetadataDTO {
            file_id: storage_metadata.file_id.clone(),
            mime_type: Some(storage_metadata.mime_type),
            size: Some(storage_metadata.size),
            user_id: if file_type == "permanent" {
                user_id.clone()
            } else {
                None
            },
            description,
            file_name: Some(filename),
            server_id: Some(app_state.server_id.clone()),
            uploaded_at: Some(Utc::now()),
            download_count: Some(0),
            last_access: Some(Utc::now()),
            delete_at,
        };
        let metadata = app_state
            .metadata_repository
            .create_metadata(metadata_dto)
            .await?;

        if file_type == "permanent" {
            if let Some(user) = user {
                let uid_str = user_id.as_ref().unwrap();
                let uid = Uuid::parse_str(uid_str).unwrap();

                let mut update_dto = UserDTO::for_update(uid);
                update_dto.file_count = Some(user.file_count + 1);
                update_dto.used_space = Some(user.used_space + file_size);
                app_state.user_repository.update_user(update_dto).await?;
            }
        }

        Ok((
            StatusCode::CREATED,
            Json(UploadFileResponse::from(metadata)),
        ))
    }

    pub async fn cleanup_expired_files(
        State(app_state): State<AppState>,
        headers: HeaderMap,
    ) -> Result<Json<CleanupResponse>, ApplicationError> {
        let provided_secret = headers
            .get("X-VK-Secret")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApplicationError::Unauthorized)?;

        let vk_secret = app_state.secrets.lock().unwrap().vk_secret.clone();
        if provided_secret != vk_secret {
            return Err(ApplicationError::Unauthorized);
        }

        let expired_files = app_state.metadata_repository.get_expired_files().await?;

        let mut deleted_count = 0;
        let mut errors = Vec::new();

        for file_metadata in expired_files {
            let delete_result = {
                let service = app_state.storage_service.get();
                service.delete(&file_metadata.file_id).await
            };

            match delete_result {
                Ok(_) => {
                    match app_state
                        .metadata_repository
                        .delete_metadata(&file_metadata.file_id)
                        .await
                    {
                        Ok(_) => {
                            if let Some(user_id_str) = file_metadata.user_id.clone() {
                                if let Ok(uid) = Uuid::parse_str(&user_id_str) {
                                    let get_user_dto = UserDTO::for_query(uid);

                                    if let Ok(user) =
                                        app_state.user_repository.get_user(get_user_dto).await
                                    {
                                        let mut update_dto = UserDTO::for_update(uid);
                                        update_dto.file_count =
                                            Some(user.file_count.saturating_sub(1));
                                        update_dto.used_space = Some(
                                            user.used_space.saturating_sub(file_metadata.size),
                                        );

                                        if let Err(e) =
                                            app_state.user_repository.update_user(update_dto).await
                                        {
                                            errors.push(format!(
                                                "Error updating user quota for file {}: {:?}",
                                                file_metadata.file_id, e
                                            ));
                                        }
                                    }
                                }
                            }

                            deleted_count += 1;
                        }
                        Err(e) => {
                            errors.push(format!(
                                "Error deleting metadata for file {}: {:?}",
                                file_metadata.file_id, e
                            ));
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!(
                        "Error deleting file {} from storage: {:?}",
                        file_metadata.file_id, e
                    ));
                }
            }
        }

        Ok(Json(CleanupResponse {
            deleted_count,
            errors,
        }))
    }

    pub async fn download_file(
        State(app_state): State<AppState>,
        Path(file_id): Path<String>,
    ) -> Result<Response, ApplicationError> {
        let metadata = app_state
            .metadata_repository
            .increment_download_count(&file_id)
            .await?;

        let file_bytes = {
            let service = app_state.storage_service.get();
            service.download(&file_id).await?
        };

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, metadata.mime_type)
            .header(header::CONTENT_LENGTH, file_bytes.len())
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", metadata.file_name),
            )
            .body(Body::from(file_bytes))
            .unwrap();

        Ok(response)
    }

    pub async fn get_file_metadata(
        State(app_state): State<AppState>,
        Path(file_id): Path<String>,
    ) -> Result<Json<FileResponse>, ApplicationError> {
        let metadata = app_state.metadata_repository.get_metadata(&file_id).await?;
        Ok(Json(FileResponse::from(metadata)))
    }

    pub async fn update_file_metadata(
        State(app_state): State<AppState>,
        Path(file_id): Path<String>,
        Json(body): Json<UpdateFileRequest>,
    ) -> Result<Json<FileResponse>, ApplicationError> {
        let current_metadata = app_state.metadata_repository.get_metadata(&file_id).await?;

        if current_metadata.user_id.is_none() {
            return Err(ApplicationError::BadRequest(
                "Cannot update metadata of temporary files".to_string(),
            ));
        }

        let update_dto = MetadataDTO {
            file_id: file_id.clone(),
            description: body.description,
            file_name: body.file_name,
            delete_at: body.delete_at,
            ..Default::default()
        };

        let updated_metadata = app_state
            .metadata_repository
            .update_metadata(update_dto)
            .await?;

        Ok(Json(FileResponse::from(updated_metadata)))
    }

    pub async fn delete_file(
        State(app_state): State<AppState>,
        Path(file_id): Path<String>,
    ) -> Result<StatusCode, ApplicationError> {
        let metadata = app_state.metadata_repository.get_metadata(&file_id).await?;

        {
            let service = app_state.storage_service.get();
            service.delete(&file_id).await?;
        }

        app_state
            .metadata_repository
            .delete_metadata(&file_id)
            .await?;

        if let Some(user_id_str) = metadata.user_id {
            if let Ok(uid) = Uuid::parse_str(&user_id_str) {
                let get_user_dto = UserDTO::for_query(uid);

                if let Ok(user) = app_state.user_repository.get_user(get_user_dto).await {
                    let mut update_dto = UserDTO::for_update(uid);
                    update_dto.file_count = Some(user.file_count.saturating_sub(1));
                    update_dto.used_space = Some(user.used_space.saturating_sub(metadata.size));
                    app_state.user_repository.update_user(update_dto).await?;
                }
            }
        }

        Ok(StatusCode::NO_CONTENT)
    }
}
