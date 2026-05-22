use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::domain::transaction::{Transaction, TransactionType};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTransactionRequest {
    pub account_id: i64,
    pub category_id: i64,
    #[schema(example = 500.00)]
    pub amount: Decimal,
    #[schema(example = "Expense")]
    pub transaction_type: TransactionTypeDto, // "Income" или "Expense"
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TransactionResponse {
    pub id: i64,
    pub account_id: i64,
    pub category_id: i64,
    pub amount: Decimal,
    pub transaction_type: TransactionTypeDto,
    pub description: Option<String>,
    pub transaction_time: NaiveDateTime,
    pub budget_exceeded: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransactionTypeDto {
    Income,
    Expense,
}
impl From<TransactionTypeDto> for TransactionType {
    fn from(value: TransactionTypeDto) -> Self {
        match value {
            TransactionTypeDto::Expense => Self::Expense,
            TransactionTypeDto::Income => Self::Income,
        }
    }
}

impl From<TransactionType> for TransactionTypeDto {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Expense => Self::Expense,
            TransactionType::Income => Self::Income,
        }
    }
}

impl TransactionResponse {
    pub fn from_transaction(value: Transaction, budget_exceeded: bool) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            category_id: value.category_id,
            amount: value.amount,
            transaction_type: value.transaction_type.into(),
            description: value.description,
            transaction_time: value.transaction_date,
            budget_exceeded,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetTransactionsRequest {
    pub account_id: i64,
    #[schema(example = "2026-05-01T00:00:00")]
    pub from: NaiveDateTime,
    #[schema(example = "2026-06-01T00:00:00")]
    pub to: NaiveDateTime,
}
