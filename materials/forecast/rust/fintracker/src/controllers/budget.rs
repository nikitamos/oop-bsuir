use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{models::domain::budget::Budget, repositories::*, shared::error::Result};
use std::sync::Arc;

pub struct BudgetController {
    budget_repo: Arc<dyn BudgetRepository>,
    category_repo: Arc<dyn CategoryRepository>,
}

impl BudgetController {
    pub fn new(
        budget_repo: Arc<dyn BudgetRepository>,
        category_repo: Arc<dyn CategoryRepository>,
    ) -> Self {
        todo!()
    }

    pub async fn set_budget(
        &self,
        user_id: i64,
        category_id: i64,
        limit: Decimal,
        month: NaiveDate,
    ) -> Result<Budget> {
        todo!()
    }
    pub async fn get_budgets(&self, user_id: i64, month: NaiveDate) -> Result<Vec<Budget>> {
        todo!()
    }
    pub async fn get_budget_status(&self, budget_id: i64) -> Result<Budget> {
        todo!()
    }
}
