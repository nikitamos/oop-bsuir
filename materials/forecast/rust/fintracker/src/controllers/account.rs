use std::sync::Arc;

use crate::{
    models::domain::account::{Account, AccountType},
    repositories::AccountRepository,
    shared::error::Result,
};
pub struct AccountController {
    account_repo: Arc<dyn AccountRepository>,
}

impl AccountController {
    pub fn new(account_repo: Arc<dyn AccountRepository>) -> Self {
        todo!()
    }

    pub async fn create_account(
        &self,
        user_id: i64,
        name: String,
        account_type: AccountType,
    ) -> Result<Account> {
        todo!()
    }
    pub async fn get_accounts(&self, user_id: i64) -> Result<Vec<Account>> {
        todo!()
    }
}
