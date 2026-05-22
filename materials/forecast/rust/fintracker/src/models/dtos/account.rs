use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::domain::account::{Account, AccountType};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAccountRequest {
    #[schema(example = "Наличные")]
    pub name: String,
    #[schema(example = "Cash")]
    pub account_type: AccountTypeDto,
    pub initial_balance: Decimal,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AccountResponse {
    pub id: i64,
    pub name: String,
    pub account_type: AccountTypeDto,
    #[schema(example = "137.42")]
    pub balance: Decimal,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        AccountResponse {
            id: account.id,
            name: account.name,
            account_type: account.account_type.into(),
            balance: account.balance,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum AccountTypeDto {
    Cash,
    BankCard,
}

impl From<AccountTypeDto> for AccountType {
    fn from(value: AccountTypeDto) -> Self {
        match value {
            AccountTypeDto::Cash => Self::Cash,
            AccountTypeDto::BankCard => Self::BankCard,
        }
    }
}

impl From<AccountType> for AccountTypeDto {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Cash => Self::Cash,
            AccountType::BankCard => Self::BankCard,
        }
    }
}
