mod error;
mod google_drive_storage;
mod supabase_storage;

pub use error::StorageError;
pub use google_drive_storage::GDriveStorageService;
pub use supabase_storage::SupabaseStorageService;

use std::sync::Arc;

use crate::{
    application::services::StorageService,
    domain::config::{local::Provider, secrets::Secrets},
};

pub fn create_storage_service(
    provider: &Provider,
    secrets: &Secrets,
) -> Result<Arc<dyn StorageService>, StorageError> {
    match provider {
        Provider::GDrive => {
            let gdrive_secrets = secrets.gdrive_secrets.as_ref().ok_or_else(|| {
                StorageError::InvalidCredentials("GDrive secrets not found".to_string())
            })?;

            let service = GDriveStorageService::new(gdrive_secrets.clone())?;
            Ok(Arc::new(service))
        }
        Provider::Supabase => {
            let supabase_secrets = secrets.supabase_secrets.as_ref().ok_or_else(|| {
                StorageError::InvalidCredentials("Supabase secrets not found".to_string())
            })?;

            let service = SupabaseStorageService::new(supabase_secrets.clone());
            Ok(Arc::new(service))
        }
    }
}
