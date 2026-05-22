use crate::models::{
    domain::account::{Account as DomainAccount, AccountType as DomainAccountType},
    entities::account::{AccountType as DbAccountType, Model as AccountEntity},
};
use crate::models::{
    domain::transaction::{Transaction as DomainTransaction, TransactionType as DomainTxType},
    entities::transaction::{Model as TxEntity, TransactionType as DbTxType},
};

impl From<AccountEntity> for DomainAccount {
    fn from(entity: AccountEntity) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            name: entity.name,
            account_type: entity.account_type.into(),
            balance: entity.balance,
            created_at: entity.created_at,
        }
    }
}

impl From<DomainAccount> for crate::models::entities::account::ActiveModel {
    fn from(domain: DomainAccount) -> Self {
        use sea_orm::{ActiveValue, Set};

        Self {
            id: ActiveValue::set(domain.id),
            user_id: Set(domain.user_id),
            name: Set(domain.name),
            account_type: Set(domain.account_type.into()),
            balance: Set(domain.balance),
            created_at: Set(domain.created_at),
        }
    }
}

impl From<DbAccountType> for DomainAccountType {
    fn from(db: DbAccountType) -> Self {
        match db {
            DbAccountType::Cash => DomainAccountType::Cash,
            DbAccountType::BankCard => DomainAccountType::BankCard,
        }
    }
}

impl From<DomainAccountType> for DbAccountType {
    fn from(domain: DomainAccountType) -> Self {
        match domain {
            DomainAccountType::Cash => DbAccountType::Cash,
            DomainAccountType::BankCard => DbAccountType::BankCard,
        }
    }
}

impl From<TxEntity> for DomainTransaction {
    fn from(entity: TxEntity) -> Self {
        Self {
            id: entity.id,
            account_id: entity.account_id,
            category_id: entity.category_id,
            amount: entity.amount,
            transaction_type: entity.transaction_type.into(),
            description: entity.description,
            transaction_date: entity.transaction_date,
        }
    }
}

impl From<DomainTxType> for DbTxType {
    fn from(domain: DomainTxType) -> Self {
        match domain {
            DomainTxType::Income => DbTxType::Income,
            DomainTxType::Expense => DbTxType::Expense,
        }
    }
}

impl From<DbTxType> for DomainTxType {
    fn from(db: DbTxType) -> Self {
        match db {
            DbTxType::Income => DomainTxType::Income,
            DbTxType::Expense => DomainTxType::Expense,
        }
    }
}
