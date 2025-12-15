use async_trait::async_trait;

use crate::{
    application::{dto::user_dto::UserDTO, error::ApplicationError},
    domain::models::user::User,
};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: UserDTO, new_space: u64) -> Result<User, ApplicationError>;
    async fn get_user(&self, user: UserDTO) -> Result<User, ApplicationError>;
    async fn update_user(&self, user: UserDTO) -> Result<User, ApplicationError>;
    async fn delete_user(&self, user: UserDTO) -> Result<User, ApplicationError>;
}
