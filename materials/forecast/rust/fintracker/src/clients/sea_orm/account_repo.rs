use async_trait::async_trait;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::{
    models::{
        self,
        domain::account::Account as DomainAccount,
        entities::account::{self, AccountType as DbAccountType, Entity as AccountEntity},
    },
    shared::error::{ApplicationError as AppError, Result},
};

use super::error::DbResultExt;

pub struct SeaOrmAccountRepository {
    db: sea_orm::DatabaseConnection,
}

impl SeaOrmAccountRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl super::AccountRepository for SeaOrmAccountRepository {
    async fn create(&self, account: &DomainAccount) -> Result<DomainAccount> {
        let active_model = account::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: Set(account.user_id),
            name: Set(account.name.clone()),
            account_type: Set(match account.account_type {
                crate::models::domain::account::AccountType::Cash => DbAccountType::Cash,
                crate::models::domain::account::AccountType::BankCard => DbAccountType::BankCard,
            }),
            balance: Set(account.balance),
            created_at: Set(account.created_at),
        };

        let inserted = active_model.insert(&self.db).await.map_db_err()?;

        Ok(inserted.into())
    }

    async fn find_by_user_id(&self, user_id: i64) -> Result<Vec<DomainAccount>> {
        let entities = AccountEntity::find()
            .filter(account::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_db_err()?;

        Ok(entities.into_iter().map(|e| e.into()).collect())
    }

    async fn find_by_id(&self, id: i64, owner_id: Option<i64>) -> Result<Option<DomainAccount>> {
        let entity = AccountEntity::find_by_id(id)
            .filter(account::Column::UserId.eq(owner_id))
            .one(&self.db)
            .await
            .map_db_err()?;

        Ok(entity.map(|e| e.into()))
    }

    async fn update_balance(&self, account_id: i64, new_balance: Decimal) -> Result<()> {
        let mut account: account::ActiveModel = AccountEntity::find_by_id(account_id)
            .one(&self.db)
            .await
            .map_db_err()?
            .ok_or(AppError::NotFound(format!(
                "No account with id {account_id}"
            )))?
            .into();

        account.balance = Set(new_balance);

        account.update(&self.db).await.map_db_err()?;

        Ok(())
    }
}
