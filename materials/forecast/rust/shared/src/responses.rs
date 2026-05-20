use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::{IntoResponses, ToResponse, ToSchema};

#[derive(Serialize, Deserialize, ToResponse, ToSchema)]
pub struct StatusResponse {
    #[schema(examples(0))]
    pub code: u16,
    pub message: String,
}

impl IntoResponse for StatusResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::from_u16(self.code).unwrap(), Json(self)).into_response()
    }
}

#[derive(Serialize, Deserialize, ToResponse, ToSchema)]
pub struct SuccessResponse<T>
where
    T: ToSchema + Serialize,
{
    #[schema(examples(200))]
    pub code: u16,
    pub message: Option<String>,
    pub data: T,
}

impl<T> SuccessResponse<T>
where
    T: ToSchema + Serialize,
{
    pub fn new(code: StatusCode, data: T) -> Self {
        Self {
            code: code.as_u16(),
            message: code.canonical_reason().map(ToString::to_string),
            data,
        }
    }
}

impl<T> IntoResponse for SuccessResponse<T>
where
    T: ToSchema + Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (StatusCode::from_u16(self.code).unwrap(), Json(self)).into_response()
    }
}
