use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "0^zU.?Ns*+I=C:c%J{\\8")]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "0^zU.?Ns*+I=C:c%J{\\8")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    #[schema(example = "eyJhbGciOiJIUzI1NiIs...")]
    pub token: String,
    #[schema(example = "user@example.com")]
    pub email: String,
}
