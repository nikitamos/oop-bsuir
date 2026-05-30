use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{FromRef, FromRequestParts, State};
use axum::http;
use axum::http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::api::error::AppError;
use crate::shared::error::{ApplicationError, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_id: i64,
    pub exp: usize,
}

#[derive(Clone)]
pub struct JwtState {
    decoding_key: DecodingKey,
    validation: Validation,
    encoding_key: EncodingKey,
}

impl JwtState {
    pub fn from_secret(secret: &[u8]) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 5;
        Self {
            decoding_key: DecodingKey::from_secret(secret),
            encoding_key: EncodingKey::from_secret(secret),
            validation,
        }
    }
    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET environment variable not set");
        Self::from_secret(secret.as_bytes())
    }
    pub fn generate_token(
        &self,
        user_id: i64,
        expires_in_secs: u64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + expires_in_secs as usize;

        let claims = Claims { user_id, exp };
        encode(&Header::default(), &claims, &self.encoding_key)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct UserIdExtractor {
    pub user_id: i64,
}

fn verify_token(token: &str, jwt_state: &JwtState) -> Result<i64> {
    let token_data = decode::<Claims>(token, &jwt_state.decoding_key, &jwt_state.validation)
        .map_err(|error| {
            tracing::warn!(message="invalid JWT token passed", verification = ?error);
            crate::shared::error::ApplicationError::BadAuthToken("Invalid auth token".to_string())
        })?;
    Ok(token_data.claims.user_id)
}

impl<S> FromRequestParts<S> for UserIdExtractor
where
    S: Send + Sync,
    JwtState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(jwt_state) = State::<JwtState>::from_request_parts(parts, state)
            .await
            .map_err(|_| ApplicationError::Unauthorized("Missing auth token".to_string()))?;

        let token = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApplicationError::Unauthorized(
                "Missing auth token".to_string(),
            ))?;

        if !token.starts_with("Bearer ") {
            return Err(ApplicationError::BadAuthToken("Invalid auth token".to_string()).into());
        }
        let token = &token[7..];

        let user_id = verify_token(token, &jwt_state)?;
        Ok(UserIdExtractor { user_id })
    }
}
