use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::{error, warn};

use crate::application::error::ApplicationError;

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApplicationError::NotFound => {
                warn!("Resource not found");
                (StatusCode::NOT_FOUND, "Resource not found".to_string())
            }
            ApplicationError::BadRequest(ref msg) => {
                warn!("Bad request: {}", msg);
                (StatusCode::BAD_REQUEST, "Bad request".to_string())
            }
            ApplicationError::Unauthorized => {
                warn!("Unauthorized access attempt");
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            ApplicationError::InvalidToken => {
                warn!("Invalid or expired upload token");
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string())
            }
            ApplicationError::PayloadTooLarge => {
                warn!("File too large");
                (StatusCode::PAYLOAD_TOO_LARGE, "File too large".to_string())
            }
            ApplicationError::InsufficientStorage => {
                warn!("Insufficient storage quota");
                (
                    StatusCode::INSUFFICIENT_STORAGE,
                    "Insufficient storage quota".to_string(),
                )
            }
            ApplicationError::InternalError(ref msg) => {
                error!("Internal server error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            ApplicationError::DatabaseError(ref msg) => {
                error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
