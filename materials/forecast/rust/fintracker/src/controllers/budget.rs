use chrono::NaiveDate;

use crate::{models::domain::budget::Budget, repositories::*, shared::error::Result};
use std::sync::Arc;

pub struct BudgetController {
    budget_repo: Arc<dyn BudgetRepository>,
    category_repo: Arc<dyn CategoryRepository>,
}

type BudgetStatus = ();
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
        limit: f64,
        month: NaiveDate,
    ) -> Result<Budget> {
        todo!()
    }
    pub async fn get_budgets(&self, user_id: i64, month: NaiveDate) -> Result<Vec<Budget>> {
        todo!()
    }
    pub async fn check_budget_status(&self, budget_id: i64) -> Result<BudgetStatus> {
        todo!()
    }
}
