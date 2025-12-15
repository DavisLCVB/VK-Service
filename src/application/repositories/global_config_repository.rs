use async_trait::async_trait;

use crate::{
    application::{dto::global_config_dto::GlobalConfigDTO, error::ApplicationError},
    domain::config::global::GlobalConfig,
};

#[async_trait]
pub trait GlobalConfigRepository: Send + Sync {
    async fn get_global_config(&self) -> Result<GlobalConfig, ApplicationError>;
    async fn upsert_global_config(
        &self,
        config: GlobalConfigDTO,
    ) -> Result<GlobalConfig, ApplicationError>;
}
