use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub account_id: i64,
    pub category_id: i64,
    #[schema(example = 500.00)]
    pub amount: Decimal,
    #[schema(example = "Expense")]
    pub transaction_type: String, // "Income" или "Expense"
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionResponse {
    pub id: i64,
    pub account_id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub amount: Decimal,
    pub transaction_type: String,
    pub description: Option<String>,
    pub transaction_date: String,
    pub budget_exceeded: bool,
}
#[derive(Debug, Deserialize, ToSchema)]
pub struct GetTransactionsRequest {
    pub account_id: i64,
    #[schema(example = "2026-05-01T00:00:00")]
    pub from: NaiveDateTime,
    #[schema(example = "2026-06-01T00:00:00")]
    pub to: NaiveDateTime,
}
