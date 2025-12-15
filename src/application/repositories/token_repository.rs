use crate::application::error::ApplicationError;
use async_trait::async_trait;

#[async_trait]
pub trait TokenRepository: Send + Sync {
    /// Genera un token de un solo uso y lo almacena en Redis
    ///
    /// # Arguments
    /// * `user_id` - ID de usuario opcional (None = token anónimo)
    /// * `ttl_seconds` - Tiempo de vida en segundos
    ///
    /// # Returns
    /// El token generado (UUID v4 string)
    async fn generate_token(
        &self,
        user_id: Option<String>,
        ttl_seconds: u64,
    ) -> Result<String, ApplicationError>;

    /// Verifica y consume un token (operación atómica de un solo uso)
    ///
    /// # Arguments
    /// * `token` - Token a verificar
    ///
    /// # Returns
    /// - Ok(Some(user_id)) si el token era válido y estaba asociado a un usuario
    /// - Ok(None) si el token era válido y era anónimo
    /// - Err(InvalidToken) si el token no existe, expiró o ya fue usado
    async fn verify_and_consume_token(
        &self,
        token: &str,
    ) -> Result<Option<String>, ApplicationError>;
}
