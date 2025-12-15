use async_trait::async_trait;

use crate::{
    application::{dto::secrets_dto::SecretsDTO, error::ApplicationError},
    domain::config::secrets::Secrets,
};

#[async_trait]
pub trait SecretsRepository: Send + Sync {
    async fn get_secrets(&self) -> Result<Secrets, ApplicationError>;
    async fn upsert_secrets(&self, secrets: SecretsDTO) -> Result<Secrets, ApplicationError>;
}
