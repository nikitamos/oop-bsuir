use crate::{models::dtos::ErrorResponse, shared::error::ApplicationError};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct AppError(pub ApplicationError, pub Option<StatusCode>);

impl From<ApplicationError> for AppError {
    fn from(err: ApplicationError) -> Self {
        AppError(err, None)
    }
}

impl AppError {
    pub fn with_code(mut self, code: StatusCode) -> Self {
        self.1 = Some(code);
        self
    }
    fn code_msg(&self, code: StatusCode, msg: String) -> (StatusCode, String) {
        (self.1.unwrap_or(code), msg)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            ApplicationError::NotFound(msg) => self.code_msg(StatusCode::NOT_FOUND, msg.clone()),
            ApplicationError::BadRequest(msg) => {
                self.code_msg(StatusCode::BAD_REQUEST, msg.clone())
            }
            ApplicationError::Unauthorized(msg) => {
                self.code_msg(StatusCode::UNAUTHORIZED, msg.clone())
            }
            ApplicationError::Conflict(msg) => self.code_msg(StatusCode::CONFLICT, msg.clone()),
            ApplicationError::Internal(msg) => {
                self.code_msg(StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
            ApplicationError::BadAuthToken(msg) => {
                self.code_msg(StatusCode::BAD_REQUEST, msg.clone())
            }
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
