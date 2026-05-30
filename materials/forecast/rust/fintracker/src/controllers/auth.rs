use crate::{
    models::domain::user::User,
    shared::error::{ApplicationError, Result},
};
use std::sync::Arc;

use crate::repositories::UserRepository;
pub struct AuthController {
    user_repo: Arc<dyn UserRepository>,
}

impl AuthController {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn register(&self, email: String, password: String) -> Result<i64> {
        let x = self
            .user_repo
            .create(
                &User::new_with_raw_password(email, password)
                    .map_err(|e| ApplicationError::Internal(e.to_string()))?,
            )
            .await?;
        Ok(x.id)
    }
    pub async fn login(&self, email: String, password: String) -> Result<i64> {
        let Some(user_model) = self.user_repo.find_by_email(&email).await? else {
            return Err(ApplicationError::NotFound("No such user".to_string()));
        };
        let user = User::verify(
            user_model.id,
            email,
            password,
            user_model.password_hash.clone(),
        )
        .ok_or_else(|| ApplicationError::BadRequest("Invalid credentials".to_string()))?;
        if user.password_hash == user_model.password_hash {
            Ok(user.id)
        } else {
            Err(ApplicationError::BadRequest(
                "Invalid credentials".to_string(),
            ))
        }
    }
}
