use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use rust_decimal::Decimal;

use crate::models::domain::transaction::{Transaction, TransactionType};
use crate::repositories::*;
use crate::shared::error::{ApplicationError, Result};
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
        Self {
            transaction_repo,
            account_repo,
            budget_repo,
            category_repo,
        }
    }

    pub async fn create_transaction(
        &self,
        account_id: i64,
        category_id: i64,
        amount: Decimal,
        transaction_type: TransactionType,
        description: Option<String>,
    ) -> Result<(Transaction, Option<String>)> {
        let mut account = self
            .account_repo
            .find_by_id(account_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::NotFound(format!("Account with id {} not found", account_id))
            })?;

        let category = self.category_repo.find_by_id(category_id).await?;

        if category.category_type != transaction_type {
            return Err(ApplicationError::BadRequest(format!(
                "Type of Category '{}' (id={}) does not match type of the transaction",
                category.name, category.id
            )));
        }

        if let TransactionType::Expense = transaction_type {
            if !account.can_withdraw(amount) {
                return Err(ApplicationError::BadRequest(format!(
                    "Insufficient funds: balance is {}, but tried to spend {}",
                    account.balance, amount
                )));
            }
        }

        let transaction =
            account.apply_transaction(amount, &category, description, Utc::now().naive_utc())?;

        let now = Utc::now().naive_utc();
        let saved_transaction = self.transaction_repo.create(&transaction).await?;
        self.account_repo
            .update_balance(account.id, account.balance)
            .await?;

        let mut warning: Option<String> = None;
        if matches!(transaction_type, TransactionType::Expense) {
            let current_month = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();

            if let Some(mut budget) = self
                .budget_repo
                .find_by_user_cat_month(account.user_id, category_id, current_month)
                .await?
            {
                budget.record_expense(amount);
                self.budget_repo
                    .update_spent(budget.id, budget.current_spent)
                    .await?;
                warning = budget.get_warning(Some(&category.name));
            }
        }

        Ok((saved_transaction, warning))
    }

    pub async fn get_transactions(
        &self,
        account_id: i64,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> Result<Vec<Transaction>> {
        self.transaction_repo
            .find_by_account(account_id, from, to)
            .await
    }
}
