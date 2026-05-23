use async_trait::async_trait;
use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Set,
};

use crate::{
    models::{
        self,
        domain::transaction::{Transaction as DomainTransaction, TransactionType as DomainTxType},
        entities::{
            account,
            transaction::{self, Entity as TransactionEntity, TransactionType as DbTxType},
        },
    },
    shared::error::{Result},
};

use super::{conversion::transaction::TransactionEntityExt, error::DbResultExt};

pub struct SeaOrmTransactionRepository {
    db: sea_orm::DatabaseConnection,
}

impl SeaOrmTransactionRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl super::TransactionRepository for SeaOrmTransactionRepository {
    async fn create(&self, transaction: &DomainTransaction) -> Result<DomainTransaction> {
        let active_model = transaction::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            account_id: Set(transaction.account_id),
            category_id: Set(transaction.category_id),
            amount: Set(transaction.amount),
            transaction_type: Set(match transaction.transaction_type {
                DomainTxType::Income => DbTxType::Income,
                DomainTxType::Expense => DbTxType::Expense,
            }),
            description: Set(transaction.description.clone()),
            transaction_date: Set(transaction.transaction_date),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };

        let inserted = active_model.insert(&self.db).await.map_db_err()?;

        Ok(inserted.into_domain())
    }

    async fn find_by_account(
        &self,
        account_id: i64,
        from: chrono::NaiveDateTime,
        to: chrono::NaiveDateTime,
    ) -> Result<Vec<DomainTransaction>> {
        let entities = TransactionEntity::find_by_id(account_id)
            .filter(
                Condition::all()
                    .add(transaction::Column::TransactionDate.gte(from))
                    .add(transaction::Column::TransactionDate.lte(to)),
            )
            .order_by_desc(transaction::Column::TransactionDate)
            .all(&self.db)
            .await
            .map_db_err()?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }

    async fn find_by_category_and_month(
        &self,
        category_id: i64,
        month: NaiveDate,
    ) -> Result<Vec<DomainTransaction>> {
        // Вычисляем границы месяца
        let start = month.and_hms_opt(0, 0, 0).unwrap();
        let end = month
            .succ_opt()
            .unwrap_or(month)
            .and_hms_opt(23, 59, 59)
            .unwrap();

        let entities = TransactionEntity::find()
            .filter(transaction::Column::CategoryId.eq(category_id))
            .filter(
                Condition::all()
                    .add(transaction::Column::TransactionDate.gte(start))
                    .add(transaction::Column::TransactionDate.lt(end)),
            )
            .all(&self.db)
            .await
            .map_db_err()?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }

    async fn find_by_category_user_period(
        &self,
        category_id: i64,
        user_id: i64,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<DomainTransaction>> {
        let from_dt = from.and_hms_opt(0, 0, 0).unwrap();
        let to_dt = to.succ_opt().unwrap_or(to).and_hms_opt(23, 59, 59).unwrap();

        let entities = TransactionEntity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                transaction::Relation::Account.def(),
            )
            .filter(transaction::Column::CategoryId.eq(category_id))
            .filter(models::entities::account::Column::UserId.eq(user_id))
            .filter(
                Condition::all()
                    .add(transaction::Column::TransactionDate.gte(from_dt))
                    .add(transaction::Column::TransactionDate.lte(to_dt)),
            )
            .all(&self.db)
            .await
            .map_db_err()?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }

    async fn find_by_user_and_period(
        &self,
        user_id: i64,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<DomainTransaction>> {
        let from_dt = from.and_hms_opt(0, 0, 0).unwrap();
        let to_dt = to.succ_opt().unwrap_or(to).and_hms_opt(23, 59, 59).unwrap();

        let entities = TransactionEntity::find()
            .join(
                sea_orm::JoinType::InnerJoin,
                transaction::Relation::Account.def(),
            )
            .filter(account::Column::UserId.eq(user_id))
            .filter(
                Condition::all()
                    .add(transaction::Column::TransactionDate.gte(from_dt))
                    .add(transaction::Column::TransactionDate.lte(to_dt)),
            )
            .order_by_desc(transaction::Column::TransactionDate)
            .all(&self.db)
            .await
            .map_db_err()?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }
}
