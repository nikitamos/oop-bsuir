use crate::shared::error::Result;
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

    pub async fn register(&self, email: String, password: String) -> Result<()> {
        todo!()
    }
    pub async fn login(&self, email: String, password: String) -> Result<()> {
        todo!()
    }
}
