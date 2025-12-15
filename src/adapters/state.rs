use axum::extract::FromRef;
use std::sync::{Arc, Mutex};

use crate::{
    adapters::storage_service_wrapper::StorageServiceWrapper,
    application::repositories::{
        global_config_repository::GlobalConfigRepository,
        local_config_repository::LocalConfigRepository, metadata_repository::MetadataRepository,
        secrets_repository::SecretsRepository, token_repository::TokenRepository,
        user_repository::UserRepository,
    },
    domain::config::{global::GlobalConfig, local::LocalConfig, secrets::Secrets},
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub server_id: String,
    pub secrets: Arc<Mutex<Secrets>>,
    pub local_config: Arc<Mutex<LocalConfig>>,
    pub global_config: Arc<Mutex<GlobalConfig>>,
    pub user_repository: Arc<dyn UserRepository>,
    pub metadata_repository: Arc<dyn MetadataRepository>,
    pub secrets_repository: Arc<dyn SecretsRepository>,
    pub global_config_repository: Arc<dyn GlobalConfigRepository>,
    pub local_config_repository: Arc<dyn LocalConfigRepository>,
    pub storage_service: StorageServiceWrapper,
    pub token_repository: Arc<dyn TokenRepository>,
}
