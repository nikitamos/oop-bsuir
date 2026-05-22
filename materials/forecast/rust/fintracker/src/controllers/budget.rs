use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{
    models::domain::budget::Budget,
    repositories::*,
    shared::{
        error::{ApplicationError, Result},
        last_day_of_month,
    },
};
use std::sync::Arc;

pub struct BudgetController {
    budget_repo: Arc<dyn BudgetRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    transaction_repo: Arc<dyn TransactionRepository>,
}

impl BudgetController {
    pub fn new(
        budget_repo: Arc<dyn BudgetRepository>,
        category_repo: Arc<dyn CategoryRepository>,
        transaction_repo: Arc<dyn TransactionRepository>,
    ) -> Self {
        Self {
            budget_repo,
            category_repo,
            transaction_repo,
        }
    }

    pub async fn set_budget(
        &self,
        user_id: i64,
        category_id: i64,
        limit: Decimal,
        month: NaiveDate,
    ) -> Result<Budget> {
        let budget = self
            .budget_repo
            .find_by_user_cat_month(user_id, category_id, month)
            .await?;
        if let Some(mut budget) = budget {
            budget.amount_limit = limit;
            self.budget_repo.update(&budget).await
        } else {
            self.create_budget(user_id, category_id, limit, month).await
        }
    }

    async fn create_budget(
        &self,
        user_id: i64,
        category_id: i64,
        limit: Decimal,
        month: NaiveDate,
    ) -> Result<Budget> {
        let cat = self.category_repo.find_by_id(category_id).await?.name;
        let current_spent: Decimal = self
            .transaction_repo
            .find_by_category_user_period(category_id, user_id, month, last_day_of_month(month))
            .await?
            .into_iter()
            .map(|x| x.amount)
            .sum();
        self.budget_repo
            .create(&Budget {
                id: 0,
                user_id,
                category_id,
                category_name: cat,
                amount_limit: limit,
                month,
                current_spent,
            })
            .await
    }
    pub async fn get_budgets(&self, user_id: i64, month: NaiveDate) -> Result<Vec<Budget>> {
        self.budget_repo.find_by_user(user_id, month).await
    }
    pub async fn get_budget_status(&self, budget_id: i64) -> Result<Budget> {
        self.budget_repo
            .find_by_id(budget_id)
            .await?
            .ok_or(ApplicationError::NotFound(
                "no budget with given id".to_string(),
            ))
    }
}
