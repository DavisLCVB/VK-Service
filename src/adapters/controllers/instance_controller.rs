use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    Json,
};
use tracing::{info, warn};

use crate::{
    adapters::storage_service_wrapper::StorageServiceWrapper,
    application::{
        dto::local_config_dto::LocalConfigDTO,
        error::ApplicationError,
        repositories::{
            global_config_repository::GlobalConfigRepository,
            local_config_repository::LocalConfigRepository, secrets_repository::SecretsRepository,
        },
    },
    domain::config::{global::GlobalConfig, local::LocalConfig, secrets::Secrets},
    services,
};

pub struct InstanceController;

impl InstanceController {
    pub async fn get_all_instances(
        State(local_config_repo): State<Arc<dyn LocalConfigRepository>>,
    ) -> Result<Json<Vec<String>>, ApplicationError> {
        info!("Getting all instance IDs");
        let instance_ids = local_config_repo.get_all_instance_ids().await?;
        Ok(Json(instance_ids))
    }

    pub async fn get_instance(
        Path(server_id): Path<String>,
        State(local_config_repo): State<Arc<dyn LocalConfigRepository>>,
    ) -> Result<Json<LocalConfig>, ApplicationError> {
        info!("Getting instance config for server_id: {}", server_id);
        let config = local_config_repo.get_local_config(&server_id).await?;
        Ok(Json(config))
    }

    pub async fn update_instance(
        Path(server_id): Path<String>,
        State(app_state_server_id): State<String>,
        State(local_config_repo): State<Arc<dyn LocalConfigRepository>>,
        State(global_config_repo): State<Arc<dyn GlobalConfigRepository>>,
        State(secrets_repo): State<Arc<dyn SecretsRepository>>,
        State(global_config_state): State<Arc<Mutex<GlobalConfig>>>,
        State(secrets_state): State<Arc<Mutex<Secrets>>>,
        State(local_config_state): State<Arc<Mutex<LocalConfig>>>,
        State(storage_service_state): State<StorageServiceWrapper>,
        Json(body): Json<LocalConfigDTO>,
    ) -> Result<Json<LocalConfig>, ApplicationError> {
        info!("Updating instance config for server_id: {}", server_id);

        // Validate that the server_id in the path matches the environment server_id
        if server_id != app_state_server_id {
            warn!(
                "Server ID mismatch: path={}, env={}",
                server_id, app_state_server_id
            );
            return Err(ApplicationError::BadRequest(
                "Invalid server ID".to_string()
            ));
        }

        // Get old provider before updating
        let old_provider = {
            let old_config = local_config_state.lock().unwrap();
            old_config.provider.clone()
        };

        // Update local config
        let local_config = local_config_repo
            .upsert_local_config(&server_id, body)
            .await?;
        *local_config_state.lock().unwrap() = local_config.clone();
        info!(
            "Local config updated successfully for server_id: {}, provider: {:?}",
            server_id, local_config.provider
        );

        // Refresh global config from database
        match global_config_repo.get_global_config().await {
            Ok(global_config) => {
                *global_config_state.lock().unwrap() = global_config.clone();
                info!(
                    "Global config refreshed successfully: max_size={}, default_quota={}",
                    global_config.max_size, global_config.default_quota
                );
            }
            Err(e) => {
                warn!("Failed to refresh global config: {:?}", e);
                return Err(e);
            }
        }

        // Refresh secrets from database
        let secrets = match secrets_repo.get_secrets().await {
            Ok(secrets) => {
                *secrets_state.lock().unwrap() = secrets.clone();
                info!("Secrets refreshed successfully: db_username={}, has_gdrive_secrets={}, has_supabase_secrets={}",
                      secrets.db_username,
                      secrets.gdrive_secrets.is_some(),
                      secrets.supabase_secrets.is_some());
                secrets
            }
            Err(e) => {
                warn!("Failed to refresh secrets: {:?}", e);
                return Err(e);
            }
        };

        // Recreate storage service if provider changed
        if old_provider != local_config.provider {
            info!(
                "Provider changed from {:?} to {:?}, recreating storage service",
                old_provider, local_config.provider
            );

            match services::create_storage_service(&local_config.provider, &secrets) {
                Ok(new_service) => {
                    storage_service_state.replace(new_service);
                    info!(
                        "Storage service recreated successfully for new provider: {:?}",
                        local_config.provider
                    );
                }
                Err(e) => {
                    warn!("Failed to recreate storage service: {:?}", e);
                    return Err(ApplicationError::InternalError(format!(
                        "Failed to create storage service for provider {:?}: {:?}",
                        local_config.provider, e
                    )));
                }
            }
        } else {
            info!(
                "Provider unchanged ({:?}), keeping existing storage service",
                local_config.provider
            );
        }

        info!(
            "Instance config update completed successfully for server_id: {}",
            server_id
        );
        Ok(Json(local_config))
    }
}
