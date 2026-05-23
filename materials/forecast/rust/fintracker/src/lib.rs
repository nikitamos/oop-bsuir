pub mod shared {
    use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};

    pub mod error {
        pub enum ApplicationError {
            NotFound(String),
            BadRequest(String),
            Unauthorized(String),
            Conflict(String),
            Internal(String),
        }

        pub type Result<T, E = ApplicationError> = std::result::Result<T, E>;
    }
    pub trait DateTimeProvider: Send + Sync {
        fn today(&self) -> NaiveDate {
            Utc::now().date_naive()
        }
        fn now(&self) -> NaiveDateTime {
            Utc::now().naive_utc()
        }
    }

    #[derive(Default)]
    pub struct DefaultDateTimeProvider;
    impl DateTimeProvider for DefaultDateTimeProvider {}
    pub fn last_day_of_month(date: NaiveDate) -> NaiveDate {
        let month = date.month();
        let year = date.year();
        let (next_year, next_month) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };

        NaiveDate::from_ymd_opt(next_year, next_month, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
    }
}
pub mod models {
    pub mod domain;
    pub mod dtos;
    pub mod entities;
    pub mod db {
        // models/db.rs
        use super::domain::*;
        use crate::{
            models::domain::{budget::Budget, transaction::Transaction},
            shared::error::Result,
        };
        use async_trait::async_trait;
        use chrono::NaiveDate;
        use rust_decimal::Decimal;
        pub trait RepositoryFactory {
            fn user_repo(&self) -> impl UserRepository;
            fn account_repo(&self) -> impl AccountRepository;
            fn category_repo(&self) -> impl CategoryRepository;
            fn transaction_repo(&self) -> impl TransactionRepository;
            fn budget_repo(&self) -> impl BudgetRepository;
        }

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
            async fn update_balance(&self, account_id: i64, new_balance: Decimal) -> Result<()>;
        }

        #[async_trait]
        pub trait CategoryRepository: Send + Sync {
            async fn find_all(&self) -> Result<Vec<category::Category>>;
            async fn find_by_id(&self, id: i64) -> Result<category::Category>;
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
            async fn find_by_category_user_period(
                &self,
                category_id: i64,
                user_id: i64,
                from: NaiveDate,
                to: NaiveDate,
            ) -> Result<Vec<transaction::Transaction>>;
            async fn find_by_user_and_period(
                &self,
                user_id: i64,
                from: NaiveDate,
                to: NaiveDate,
            ) -> Result<Vec<Transaction>>;
        }

        #[async_trait]
        pub trait BudgetRepository: Send + Sync {
            async fn create(&self, budget: &budget::Budget) -> Result<budget::Budget>;
            async fn find_by_user_cat_month(
                &self,
                user_id: i64,
                cat_id: i64,
                month: chrono::NaiveDate,
            ) -> Result<Option<budget::Budget>>;
            async fn find_by_user(&self, user_id: i64, month: NaiveDate) -> Result<Vec<Budget>>;
            async fn update_spent(&self, budget_id: i64, spent: Decimal) -> Result<()>;
            async fn update(&self, budget: &Budget) -> Result<budget::Budget>;
            async fn find_by_id(&self, id: i64) -> Result<Option<budget::Budget>>;
        }
    }
}

pub use models::db as repositories;

pub mod clients {
    pub mod sea_orm;
}

pub mod api;
pub mod controllers;
