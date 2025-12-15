use std::sync::{Arc, RwLock};

use crate::application::services::StorageService;

#[derive(Clone)]
pub struct StorageServiceWrapper {
    service: Arc<RwLock<Arc<dyn StorageService>>>,
}

impl StorageServiceWrapper {
    pub fn new(service: Arc<dyn StorageService>) -> Self {
        Self {
            service: Arc::new(RwLock::new(service)),
        }
    }

    pub fn get(&self) -> Arc<dyn StorageService> {
        self.service.read().unwrap().clone()
    }

    pub fn replace(&self, new_service: Arc<dyn StorageService>) {
        let mut service = self.service.write().unwrap();
        *service = new_service;
    }
}
