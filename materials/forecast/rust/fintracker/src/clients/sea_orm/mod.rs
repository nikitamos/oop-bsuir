use crate::models::db::*;

mod conversion;
mod error {
    use crate::shared::error::{ApplicationError as AppError, Result};
    use sea_orm::DbErr;

    pub trait DbResultExt<T> {
        fn map_db_err(self) -> Result<T>;
    }

    impl<T> DbResultExt<T> for std::result::Result<T, DbErr> {
        fn map_db_err(self) -> Result<T> {
            self.map_err(|e| match e {
                DbErr::RecordNotFound(_) => AppError::NotFound("entry not found".to_string()),
                _ => AppError::Internal(format!("DB error: {}", e)),
            })
        }
    }
}

pub mod account_repo;
pub mod budget_repo;
pub mod category_repo;
pub mod transaction_repo;
pub mod user_repo;
