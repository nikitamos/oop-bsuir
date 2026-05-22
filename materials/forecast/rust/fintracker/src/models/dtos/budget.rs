use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
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
