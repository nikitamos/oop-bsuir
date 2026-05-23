use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::{
    models::{
        domain::budget::Budget as DomainBudget,
        entities::budget::{self, Entity as BudgetEntity},
    },
    shared::error::{ApplicationError as AppError, Result},
};

use super::{conversion::budget::BudgetDomainExt, error::DbResultExt};

pub struct SeaOrmBudgetRepository {
    db: sea_orm::DatabaseConnection,
}

impl SeaOrmBudgetRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl super::BudgetRepository for SeaOrmBudgetRepository {
    async fn create(&self, budget: &DomainBudget) -> Result<DomainBudget> {
        let active_model = budget::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: Set(budget.user_id),
            category_id: Set(budget.category_id),
            amount_limit: Set(budget.amount_limit),
            month: Set(budget.month),
            current_spent: Set(Decimal::ZERO), // Новый бюджет начинается с 0
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };

        let inserted = active_model.insert(&self.db).await.map_db_err()?;

        Ok(inserted.into_domain())
    }

    async fn find_by_user_cat_month(
        &self,
        user_id: i64,
        cat_id: i64,
        month: chrono::NaiveDate,
    ) -> Result<Option<DomainBudget>> {
        let entity = BudgetEntity::find()
            .filter(budget::Column::UserId.eq(user_id))
            .filter(budget::Column::CategoryId.eq(cat_id))
            .filter(budget::Column::Month.eq(month))
            .one(&self.db)
            .await
            .map_db_err()?;

        Ok(entity.map(|e| e.into_domain()))
    }

    async fn find_by_user(&self, user_id: i64, month: NaiveDate) -> Result<Vec<DomainBudget>> {
        let entities = BudgetEntity::find()
            .filter(budget::Column::UserId.eq(user_id))
            .filter(budget::Column::Month.eq(month))
            .all(&self.db)
            .await
            .map_db_err()?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }

    async fn update_spent(&self, budget_id: i64, spent: Decimal) -> Result<()> {
        let mut budget: budget::ActiveModel = BudgetEntity::find_by_id(budget_id)
            .one(&self.db)
            .await
            .map_db_err()?
            .ok_or(AppError::NotFound(format!(
                "Budget with id {budget_id} does not exist"
            )))?
            .into();

        budget.current_spent = Set(spent);
        budget.updated_at = Set(chrono::Utc::now().naive_utc());

        budget.update(&self.db).await.map_db_err()?;

        Ok(())
    }

    async fn update(&self, budget: &DomainBudget) -> Result<DomainBudget> {
        let mut active_model: budget::ActiveModel = budget.clone().into();
        active_model.updated_at = Set(chrono::Utc::now().naive_utc());

        let updated = active_model.update(&self.db).await.map_db_err()?;

        Ok(updated.into_domain())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<DomainBudget>> {
        let entity = BudgetEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_db_err()?;

        Ok(entity.map(|e| e.into_domain()))
    }
}
