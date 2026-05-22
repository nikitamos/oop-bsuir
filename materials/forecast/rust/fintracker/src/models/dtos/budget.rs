use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::domain::budget::Budget;
#[derive(Debug, Deserialize, ToSchema)]
pub struct SetBudgetRequest {
    pub category_id: i64,
    #[schema(example = 15000.00)]
    pub amount_limit: Decimal,
    pub month: NaiveDate,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BudgetResponse {
    pub id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub amount_limit: Decimal,
    pub current_spent: Decimal,
    pub limit_exceeded: bool,
    pub month: NaiveDate,
}

impl From<Budget> for BudgetResponse {
    fn from(budget: Budget) -> Self {
        Self {
            limit_exceeded: budget.is_limit_exceeded(),
            id: budget.id,
            category_id: budget.category_id,
            category_name: budget.category_name,
            amount_limit: budget.amount_limit,
            current_spent: budget.current_spent,
            month: budget.month,
        }
    }
}
