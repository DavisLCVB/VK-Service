use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::{Arc, Mutex};
use tracing::warn;

use crate::domain::config::secrets::Secrets;

/// Middleware to validate the X-KV-SECRET header
pub async fn validate_kv_secret(
    State(secrets): State<Arc<Mutex<Secrets>>>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Response {
    let expected_secret = {
        let secrets_guard = secrets.lock().unwrap();
        secrets_guard.vk_secret.clone()
    };

    match headers.get("X-KV-SECRET") {
        Some(header_value) => {
            match header_value.to_str() {
                Ok(provided_secret) => {
                    if provided_secret == expected_secret {
                        // Secret is valid, continue to the handler
                        next.run(request).await
                    } else {
                        // Secret doesn't match - log details but return generic error
                        warn!("Invalid secret provided in X-KV-SECRET header");
                        (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
                    }
                }
                Err(_) => {
                    // Header value is not valid UTF-8 - log details but return generic error
                    warn!("X-KV-SECRET header contains invalid UTF-8");
                    (StatusCode::BAD_REQUEST, "Bad request").into_response()
                }
            }
        }
        None => {
            // Header is missing - log details but return generic error
            warn!("X-KV-SECRET header is missing");
            (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
        }
    }
}
