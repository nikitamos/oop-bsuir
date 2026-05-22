use chrono::NaiveDateTime;

use crate::models::domain::transaction::{Transaction, TransactionType};
use crate::repositories::*;
use crate::shared::error::Result;
use std::sync::Arc;

pub struct TransactionController {
    transaction_repo: Arc<dyn TransactionRepository>,
    account_repo: Arc<dyn AccountRepository>,
    budget_repo: Arc<dyn BudgetRepository>,
    category_repo: Arc<dyn CategoryRepository>,
}

impl TransactionController {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        account_repo: Arc<dyn AccountRepository>,
        budget_repo: Arc<dyn BudgetRepository>,
        category_repo: Arc<dyn CategoryRepository>,
    ) -> Self {
        todo!()
    }

    pub async fn create_transaction(
        &self,
        account_id: i64,
        category_id: i64,
        amount: f64,
        transaction_type: TransactionType,
        description: Option<String>,
    ) -> Result<Transaction> {
        todo!()
    }

    pub async fn get_transactions(
        &self,
        account_id: i64,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> Result<Vec<Transaction>> {
        todo!()
    }
}
