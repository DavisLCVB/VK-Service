use axum::{extract::State, Json};
use serde::Serialize;
use sysinfo::System;
use tracing::info;

use crate::adapters::state::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
    #[serde(rename = "serverName")]
    pub server_name: String,
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    pub provider: String,
    pub config: HealthConfigInfo,
    pub metrics: SystemMetrics,
}

#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    #[serde(rename = "cpuUsagePercent")]
    pub cpu_usage_percent: f32,
    #[serde(rename = "memoryUsedBytes")]
    pub memory_used_bytes: u64,
    #[serde(rename = "memoryTotalBytes")]
    pub memory_total_bytes: u64,
    #[serde(rename = "memoryUsagePercent")]
    pub memory_usage_percent: f32,
}

#[derive(Debug, Serialize)]
pub struct HealthConfigInfo {
    #[serde(rename = "maxSize")]
    pub max_size: u64,
    #[serde(rename = "defaultQuota")]
    pub default_quota: u64,
    #[serde(rename = "tempFileLife")]
    pub temp_file_life: u64,
    #[serde(rename = "allowedMimeTypes")]
    pub allowed_mime_types: Vec<String>,
}

pub struct HealthController;

impl HealthController {
    /// Health check endpoint - exclusive for VK-Gateway
    /// GET /api/v1/health
    pub async fn health_check(State(app_state): State<AppState>) -> Json<HealthResponse> {
        info!("Health check requested");

        let (server_name, server_url, provider) = {
            let local_config = app_state.local_config.lock().unwrap();
            (
                local_config.server_name.clone(),
                local_config.server_url.clone(),
                format!("{:?}", local_config.provider),
            )
        };

        let config_info = {
            let global_config = app_state.global_config.lock().unwrap();
            HealthConfigInfo {
                max_size: global_config.max_size,
                default_quota: global_config.default_quota,
                temp_file_life: global_config.temp_file_life,
                allowed_mime_types: global_config.mime_types.clone(),
            }
        };

        // Collect system metrics (optimized - only refresh what's needed)
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let cpu_usage = sys.global_cpu_usage();
        let memory_used = sys.used_memory();
        let memory_total = sys.total_memory();
        let memory_usage_percent = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        let metrics = SystemMetrics {
            cpu_usage_percent: cpu_usage,
            memory_used_bytes: memory_used,
            memory_total_bytes: memory_total,
            memory_usage_percent,
        };

        Json(HealthResponse {
            status: "healthy".to_string(),
            server_id: app_state.server_id.clone(),
            server_name,
            server_url,
            provider,
            config: config_info,
            metrics,
        })
    }
}
