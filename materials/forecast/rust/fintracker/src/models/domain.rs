use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;

pub struct User {
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

pub enum AccountType {
    Cash,
    BankCard,
    Savings,
}
pub struct Account {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub account_type: AccountType,
    pub balance: Decimal,
    pub created_at: NaiveDateTime,
}
pub struct Category {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub category_type: CategoryType,
}
pub enum CategoryType {
    Income,
    Expense,
}
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
pub struct Budget {
    pub id: i64,
    pub user_id: i64,
    pub category_id: i64,
    pub amount_limit: Decimal,
    pub month: NaiveDate,
    pub current_spent: Decimal,
}
