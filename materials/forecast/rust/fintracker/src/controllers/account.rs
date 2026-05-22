use std::sync::Arc;

use chrono::NaiveDateTime;
use rust_decimal::Decimal;

use crate::{
    models::domain::account::{Account, AccountType},
    repositories::AccountRepository,
    shared::{DateTimeProvider, DefaultDateTimeProvider, error::Result},
};
pub struct AccountController<DT: DateTimeProvider = DefaultDateTimeProvider> {
    account_repo: Arc<dyn AccountRepository>,
    date: DT,
}

impl AccountController {
    pub fn new(account_repo: Arc<dyn AccountRepository>) -> Self {
        Self {
            account_repo,
            date: Default::default(),
        }
    }
}

impl<DT: DateTimeProvider> AccountController<DT> {
    pub async fn create_account(
        &self,
        user_id: i64,
        name: String,
        account_type: AccountType,
        initial_balance: Decimal,
    ) -> Result<Account> {
        self.account_repo
            .create(&Account {
                id: 0,
                user_id,
                name,
                account_type,
                balance: initial_balance,
                created_at: self.date.now(),
            })
            .await
    }
    pub async fn get_accounts(&self, user_id: i64) -> Result<Vec<Account>> {
        self.account_repo.find_by_user_id(user_id).await
    }
}
