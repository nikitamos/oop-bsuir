use crate::{
    models::domain::user::User,
    shared::error::{ApplicationError, Result},
};
use std::sync::Arc;

use crate::repositories::UserRepository;
pub struct AuthController {
    user_repo: Arc<dyn UserRepository>,
    jwt_secret: String,
}

impl AuthController {
    pub fn new(user_repo: Arc<dyn UserRepository>, jwt_secret: String) -> Self {
        Self {
            user_repo,
            jwt_secret,
        }
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
    pub async fn login(&self, email: String, password: String) -> Result<()> {
        todo!()
    }
}
