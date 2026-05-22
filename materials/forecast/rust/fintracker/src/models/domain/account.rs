use chrono::NaiveDateTime;

use rust_decimal::Decimal;

use crate::{
    controllers::{budget, category},
    models::domain::{
        budget::Budget,
        category::Category,
        transaction::{Transaction, TransactionType},
    },
    shared::error::ApplicationError,
};

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

impl Account {
    pub fn can_withdraw(&self, amount: Decimal) -> bool {
        self.balance >= amount
    }

    pub fn apply_transaction(
        &mut self,
        amount: Decimal,
        category: &Category,
        description: Option<String>,
        transaction_date: NaiveDateTime,
    ) -> crate::shared::error::Result<Transaction> {
        match category.category_type {
            TransactionType::Income => {
                let tx =
                    Transaction::new(self.id, category, amount, description, transaction_date)?;
                self.balance += amount;
                Ok(tx)
            }
            TransactionType::Expense => {
                if !self.can_withdraw(amount) {
                    return Err(ApplicationError::BadRequest(format!(
                        "Insufficient funds: balance {}, required {}",
                        self.balance, amount
                    )));
                }
                let tx =
                    Transaction::new(self.id, category, amount, description, transaction_date)?;
                self.balance -= amount;
                Ok(tx)
            }
        }
    }
}
