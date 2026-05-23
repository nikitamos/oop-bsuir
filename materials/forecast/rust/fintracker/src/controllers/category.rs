use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{
    models::domain::{
        budget::Budget,
        category::{Category, CategoryType},
    },
    repositories::*,
    shared::error::Result,
};
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

    pub async fn create_category(
        &self,
        user_id: i64,
        name: String,
        category_type: CategoryType,
    ) -> Result<i64> {
        let cat = Category {
            id: 0,
            user_id,
            name,
            category_type: CategoryType::Expense,
        };
        self.category_repo.create(cat).await
    }
}
