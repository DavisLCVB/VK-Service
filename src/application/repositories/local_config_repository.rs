use async_trait::async_trait;

use crate::{
    application::{dto::local_config_dto::LocalConfigDTO, error::ApplicationError},
    domain::config::local::LocalConfig,
};

#[async_trait]
pub trait LocalConfigRepository: Send + Sync {
    async fn get_local_config(&self, server_id: &str) -> Result<LocalConfig, ApplicationError>;
    async fn upsert_local_config(
        &self,
        server_id: &str,
        config: LocalConfigDTO,
    ) -> Result<LocalConfig, ApplicationError>;
    async fn get_all_instance_ids(&self) -> Result<Vec<String>, ApplicationError>;
}
