use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{models::domain::{budget::Budget, category::Category}, repositories::*, shared::error::Result};
use std::sync::Arc;

pub struct CategoryController {
    budget_repo: Arc<dyn BudgetRepository>,
    category_repo: Arc<dyn CategoryRepository>,
}

impl CategoryController {
    pub fn new(
        budget_repo: Arc<dyn BudgetRepository>,
        category_repo: Arc<dyn CategoryRepository>,
    ) -> Self {
        Self {
            budget_repo,
            category_repo,
        }
    }

    pub async fn get_all_categories(&self) -> Result<Vec<Category>> {
        self.category_repo.find_all().await
    }
}
