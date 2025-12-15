mod adapters;
mod application;
mod domain;
mod services;

use std::sync::{Arc, Mutex};

use adapters::{
    controllers::{
        file_controller::FileController, health_controller::HealthController,
        instance_controller::InstanceController, user_controller::UserController,
    },
    middleware::validate_kv_secret,
    repositories::{
        PgGlobalConfigRepository, PgLocalConfigRepository, PgMetadataRepository,
        PgSecretsRepository, PgUserRepository, RedisTokenRepository,
    },
    state::AppState,
    storage_service_wrapper::StorageServiceWrapper,
};
use application::{
    dto::local_config_dto::LocalConfigDTO,
    repositories::{
        global_config_repository::GlobalConfigRepository,
        local_config_repository::LocalConfigRepository, metadata_repository::MetadataRepository,
        secrets_repository::SecretsRepository, token_repository::TokenRepository,
        user_repository::UserRepository,
    },
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize AWS SDK crypto provider (required for aws-sdk-s3)
    // This must be called before any AWS SDK operations
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();


    let server_id =
        std::env::var("SERVER_ID").expect("ERROR: SERVER_ID environment variable must be set");

    let database_url = std::env::var("DATABASE_URL")
        .expect("ERROR: DATABASE_URL environment variable must be set");

    let redis_url =
        std::env::var("REDIS_URL").expect("ERROR: REDIS_URL environment variable must be set");

    tracing::info!("Starting vk-service with SERVER_ID: {}", server_id);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");

    // Configure CORS
    let cors = if let Ok(allowed_origins) = std::env::var("CORS_ALLOWED_ORIGINS") {
        // Parse comma-separated origins
        let origins: Vec<_> = allowed_origins
            .split(',')
            .map(|s| s.trim().parse().expect("Invalid CORS origin"))
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        // Allow all origins if not specified (only for development)
        CorsLayer::permissive()
    };

    // Connect to PostgreSQL and Redis in parallel for faster startup
    tracing::info!("Connecting to databases...");
    let (pool, redis_conn_manager) = tokio::join!(
        async {
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(std::time::Duration::from_secs(30))
                .connect(&database_url)
                .await
                .expect("ERROR: Failed to connect to PostgreSQL database. Check DATABASE_URL and network connectivity.")
        },
        async {
            let redis_client = redis::Client::open(redis_url.as_str())
                .expect("ERROR: Failed to create Redis client. Check REDIS_URL format.");
            redis::aio::ConnectionManager::new(redis_client)
                .await
                .expect(
                    "ERROR: Failed to connect to Redis. Check REDIS_URL and network connectivity.",
                )
        }
    );
    tracing::info!("Database connections established");

    // Initialize repositories
    let secrets_repo =
        Arc::new(PgSecretsRepository::new(pool.clone())) as Arc<dyn SecretsRepository>;
    let global_config_repo =
        Arc::new(PgGlobalConfigRepository::new(pool.clone())) as Arc<dyn GlobalConfigRepository>;
    let local_config_repo =
        Arc::new(PgLocalConfigRepository::new(pool.clone())) as Arc<dyn LocalConfigRepository>;

    // Load all configurations in parallel for faster startup
    tracing::info!("Loading configurations from database...");
    let (local_config_result, secrets_result, global_config_result) = tokio::join!(
        local_config_repo.get_local_config(&server_id),
        secrets_repo.get_secrets(),
        global_config_repo.get_global_config()
    );
    tracing::info!("Configuration loading complete");

    // Handle local config: create with defaults if not found
    let local_config = match local_config_result {
        Ok(config) => {
            tracing::info!("Loaded existing local config for server {}", server_id);
            config
        }
        Err(_) => {
            tracing::info!(
                "Local config not found, creating default config for server {}",
                server_id
            );
            local_config_repo
                .upsert_local_config(&server_id, LocalConfigDTO::default())
                .await
                .expect("Failed to create default local config")
        }
    };

    let secrets = secrets_result.expect("Failed to load secrets");
    let global_config = global_config_result.expect("Failed to load global config");

    // Create storage service and token repository in parallel
    let (storage_service, token_repo) = tokio::join!(
        async {
            services::create_storage_service(&local_config.provider, &secrets)
                .await
                .expect("Failed to create storage service")
        },
        async {
            Arc::new(RedisTokenRepository::new(redis_conn_manager)) as Arc<dyn TokenRepository>
        }
    );

    let app_state = AppState {
        server_id,
        secrets: Arc::new(Mutex::new(secrets)),
        local_config: Arc::new(Mutex::new(local_config)),
        global_config: Arc::new(Mutex::new(global_config)),
        user_repository: Arc::new(PgUserRepository::new(pool.clone())) as Arc<dyn UserRepository>,
        metadata_repository: Arc::new(PgMetadataRepository::new(pool))
            as Arc<dyn MetadataRepository>,
        secrets_repository: secrets_repo,
        global_config_repository: global_config_repo,
        local_config_repository: local_config_repo,
        storage_service: StorageServiceWrapper::new(storage_service),
        token_repository: token_repo,
    };

    // Protected routes that require X-KV-SECRET header
    let protected_routes = Router::new()
        .route("/api/v1/health", get(HealthController::health_check))
        .route(
            "/api/v1/instances",
            get(InstanceController::get_all_instances),
        )
        .route(
            "/api/v1/instances/{server_id}",
            get(InstanceController::get_instance).patch(InstanceController::update_instance),
        )
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            validate_kv_secret,
        ));

    // Public routes that don't require authentication
    let public_routes = Router::new()
        .route("/", get(hello_world))
        .route("/api/v1/users", post(UserController::create_user))
        .route(
            "/api/v1/users/{user_id}",
            get(UserController::get_user)
                .patch(UserController::update_user)
                .delete(UserController::delete_user),
        )
        .route(
            "/api/v1/users/{user_id}/files",
            get(UserController::get_user_files),
        )
        .route(
            "/api/v1/files/token",
            post(FileController::generate_upload_token),
        )
        .route(
            "/api/v1/files",
            post(FileController::upload_file).delete(FileController::cleanup_expired_files),
        )
        .route(
            "/api/v1/files/{file_id}/content",
            get(FileController::download_file),
        )
        .route(
            "/api/v1/files/{file_id}",
            get(FileController::get_file_metadata)
                .patch(FileController::update_file_metadata)
                .delete(FileController::delete_file),
        );

    // Combine routes and add CORS layer
    let router = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        .layer(cors)
        .with_state(app_state);

    // Start the server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    tracing::info!("Server listening on 0.0.0.0:{}", port);

    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}
