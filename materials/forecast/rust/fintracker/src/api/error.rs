use crate::{models::dtos::ErrorResponse, shared::error::ApplicationError};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct AppError(pub ApplicationError);

impl From<ApplicationError> for AppError {
    fn from(err: ApplicationError) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            ApplicationError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApplicationError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApplicationError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            ApplicationError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ApplicationError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApplicationError::BadAuthToken(msg) => (StatusCode::BAD_REQUEST, msg.clone())
        };

        (
            status,
            Json(ErrorResponse {
                error: status.canonical_reason().unwrap_or("Unknown").to_string(),
                message,
            }),
        )
            .into_response()
    }
}
