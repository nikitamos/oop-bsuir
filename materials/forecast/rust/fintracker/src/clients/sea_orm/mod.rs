use crate::models::{
    db::*,
    entities::{account, budget, category, transaction, user},
};

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

pub mod prelude {
    pub use super::account_repo::SeaOrmAccountRepository;
    pub use super::budget_repo::SeaOrmBudgetRepository;
    pub use super::category_repo::SeaOrmCategoryRepository;
    pub use super::transaction_repo::SeaOrmTransactionRepository;
    pub use super::user_repo::SeaOrmUserRepository;
}

use prelude::*;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Schema};
pub struct SeaOrmRepositoryFactory {
    db: sea_orm::DatabaseConnection,
}
impl SeaOrmRepositoryFactory {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

impl RepositoryFactory for SeaOrmRepositoryFactory {
    fn user_repo(&self) -> impl UserRepository + 'static {
        SeaOrmUserRepository::new(self.db.clone())
    }
    fn account_repo(&self) -> impl AccountRepository + 'static {
        SeaOrmAccountRepository::new(self.db.clone())
    }
    fn category_repo(&self) -> impl CategoryRepository + 'static {
        SeaOrmCategoryRepository::new(self.db.clone())
    }
    fn transaction_repo(&self) -> impl TransactionRepository + 'static {
        SeaOrmTransactionRepository::new(self.db.clone())
    }
    fn budget_repo(&self) -> impl BudgetRepository + 'static {
        SeaOrmBudgetRepository::new(self.db.clone())
    }
}

pub async fn setup_db(db: &DatabaseConnection) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);
    let stmts = [
        builder.build(&schema.create_table_from_entity(user::Entity)),
        builder.build(&schema.create_table_from_entity(account::Entity)),
        builder.build(&schema.create_table_from_entity(category::Entity)),
        builder.build(&schema.create_table_from_entity(transaction::Entity)),
        builder.build(&schema.create_table_from_entity(budget::Entity)),
    ];
    for stmt in stmts {
        db.execute(stmt).await?;
    }
    Ok(())
}
