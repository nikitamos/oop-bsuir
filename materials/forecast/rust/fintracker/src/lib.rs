pub mod shared {
    pub mod error {
        pub enum ApplicationError {
            NotFound(String),
            BadRequest(String),
            Unauthorized(String),
            Conflict(String),
            Internal(String),
        }

        pub type Result<T> = std::result::Result<T, ApplicationError>;
    }
}
pub mod models {
    pub mod domain;
    pub mod dtos;
    pub mod db {
        // models/repository.rs
        use super::domain::*;
        use crate::shared::error::Result;
        use async_trait::async_trait;

        #[async_trait]
        pub trait UserRepository: Send + Sync {
            async fn find_by_email(&self, email: &str) -> Result<Option<user::User>>;
            async fn create(&self, user: &user::User) -> Result<user::User>;
            async fn find_by_id(&self, id: i64) -> Result<Option<user::User>>;
        }

        #[async_trait]
        pub trait AccountRepository: Send + Sync {
            async fn create(&self, account: &account::Account) -> Result<account::Account>;
            async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<account::Account>>;
            async fn find_by_id(&self, id: i64) -> Result<Option<account::Account>>;
            async fn update_balance(&self, account_id: i64, delta: f64) -> Result<()>;
        }

        #[async_trait]
        pub trait CategoryRepository: Send + Sync {
            async fn find_all(&self) -> Result<Vec<category::Category>>;
            async fn find_by_id(&self, id: i64) -> Result<Option<category::Category>>;
        }

        #[async_trait]
        pub trait TransactionRepository: Send + Sync {
            async fn create(
                &self,
                transaction: &transaction::Transaction,
            ) -> Result<transaction::Transaction>;
            async fn find_by_account(
                &self,
                account_id: i64,
                from: chrono::NaiveDateTime,
                to: chrono::NaiveDateTime,
            ) -> Result<Vec<transaction::Transaction>>;
            async fn find_by_category_and_month(
                &self,
                category_id: i64,
                month: chrono::NaiveDate,
            ) -> Result<Vec<transaction::Transaction>>;
        }

        #[async_trait]
        pub trait BudgetRepository: Send + Sync {
            async fn create(&self, budget: &budget::Budget) -> Result<budget::Budget>;
            async fn find_by_user_and_month(
                &self,
                user_id: i64,
                month: chrono::NaiveDate,
            ) -> Result<Vec<budget::Budget>>;
            async fn update_spent(&self, budget_id: i64, spent: f64) -> Result<()>;
            async fn find_by_id(&self, id: i64) -> Result<Option<budget::Budget>>;
        }
    }
}

pub use models::db as repositories;

pub mod clients {
    pub mod sea_orm {
        // DBs
    }
}

pub mod controllers;
