use chrono::NaiveDateTime;

use rust_decimal::Decimal;

pub enum TransactionType {
    Income,
    Expense,
}

pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub category_id: i64,
    pub amount: Decimal,
    pub transaction_type: TransactionType,
    pub description: Option<String>,
    pub transaction_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
}
