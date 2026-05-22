use chrono::NaiveDateTime;

use rust_decimal::Decimal;

pub enum AccountType {
    Cash,
    BankCard,
}

pub struct Account {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub account_type: AccountType,
    pub balance: Decimal,
    pub created_at: NaiveDateTime,
}
