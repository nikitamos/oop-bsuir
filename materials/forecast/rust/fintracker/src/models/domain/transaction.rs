use chrono::NaiveDateTime;

use rust_decimal::Decimal;

use crate::{
    models::domain::category::{Category, CategoryType},
    shared::error::ApplicationError,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
}

impl Transaction {
    pub fn new(
        account_id: i64,
        category: &Category,
        amount: Decimal,
        description: Option<String>,
        transaction_date: NaiveDateTime,
    ) -> crate::shared::error::Result<Self> {
        if amount <= Decimal::ZERO {
            return Err(ApplicationError::BadRequest(
                "Transaction amount must be positive".to_string(),
            ));
        }

        if let Some(ref desc) = description {
            if desc.trim().is_empty() {
                return Err(ApplicationError::BadRequest(
                    "Description cannot be empty string".to_string(),
                ));
            }
        }

        let transaction_type = category.category_type.into();

        Ok(Self {
            id: 0,
            account_id,
            category_id: category.id,
            amount,
            transaction_type,
            description: description.filter(|d| !d.trim().is_empty()),
            transaction_date,
        })
    }
}
