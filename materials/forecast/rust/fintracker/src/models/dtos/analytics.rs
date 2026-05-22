use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AnalyticsRequest {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategorySummary {
    pub category_id: i64,
    pub category_name: String,
    pub total_amount: Decimal,
    pub percentage: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExpensesSummary {
    pub total_expenses: Decimal,
    pub categories: Vec<CategorySummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MonthlySummary {
    pub month: NaiveDate,
    pub total_income: Decimal,
    pub total_expenses: Decimal,
    pub net: Decimal,
}
