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

async fn hello_world() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() {
    // Load .env file in development (optional in production where env vars are set directly)
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    let server_id = std::env::var("SERVER_ID").expect("SERVER_ID must be set");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let redis_client =
        redis::Client::open(redis_url.as_str()).expect("Failed to create Redis client");
    let redis_conn_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");

    let secrets_repo =
        Arc::new(PgSecretsRepository::new(pool.clone())) as Arc<dyn SecretsRepository>;
    let global_config_repo =
        Arc::new(PgGlobalConfigRepository::new(pool.clone())) as Arc<dyn GlobalConfigRepository>;
    let local_config_repo =
        Arc::new(PgLocalConfigRepository::new(pool.clone())) as Arc<dyn LocalConfigRepository>;

    // Startup upsert with default local config
    local_config_repo
        .upsert_local_config(&server_id, LocalConfigDTO::default())
        .await
        .expect("Failed to initialize local config");

    let secrets = secrets_repo
        .get_secrets()
        .await
        .expect("Failed to load secrets");
    let global_config = global_config_repo
        .get_global_config()
        .await
        .expect("Failed to load global config");
    let local_config = local_config_repo
        .get_local_config(&server_id)
        .await
        .expect("Failed to load local config");

    let storage_service = services::create_storage_service(&local_config.provider, &secrets)
        .expect("Failed to create storage service");

    let token_repo =
        Arc::new(RedisTokenRepository::new(redis_conn_manager)) as Arc<dyn TokenRepository>;

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

    // Combine routes
    let router = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
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
