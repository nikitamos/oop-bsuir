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

pub mod transaction {
    use crate::models::{
        domain::transaction::{Transaction as DomainTransaction, TransactionType as DomainTxType},
        entities::transaction::{Model as TransactionEntity, TransactionType as DbTxType},
    };

    pub trait TransactionEntityExt {
        fn into_domain(self) -> DomainTransaction;
    }

    impl TransactionEntityExt for TransactionEntity {
        fn into_domain(self) -> DomainTransaction {
            DomainTransaction {
                id: self.id,
                account_id: self.account_id,
                category_id: self.category_id,
                amount: self.amount,
                transaction_type: match self.transaction_type {
                    DbTxType::Income => DomainTxType::Income,
                    DbTxType::Expense => DomainTxType::Expense,
                },
                description: self.description,
                transaction_date: self.transaction_date,
            }
        }
    }

    impl From<DomainTransaction> for crate::models::entities::transaction::ActiveModel {
        fn from(domain: DomainTransaction) -> Self {
            use sea_orm::{ActiveValue, Set};

            Self {
                id: if domain.id > 0 {
                    ActiveValue::Set(domain.id)
                } else {
                    ActiveValue::NotSet
                },
                account_id: Set(domain.account_id),
                category_id: Set(domain.category_id),
                amount: Set(domain.amount),
                transaction_type: Set(domain.transaction_type.into()),
                description: Set(domain.description),
                transaction_date: Set(domain.transaction_date),
                created_at: Set(chrono::Utc::now().naive_utc()),
            }
        }
    }
}

pub mod budget {
    use crate::models::{
        domain::budget::Budget as DomainBudget, entities::budget::Model as BudgetEntity,
    };

    pub trait BudgetDomainExt {
        fn into_domain(self) -> DomainBudget;
    }

    impl BudgetDomainExt for BudgetEntity {
        fn into_domain(self) -> DomainBudget {
            DomainBudget {
                id: self.id,
                user_id: self.user_id,
                category_id: self.category_id,
                category_name: String::new(),
                amount_limit: self.amount_limit,
                month: self.month,
                current_spent: self.current_spent,
            }
        }
    }

    impl From<DomainBudget> for crate::models::entities::budget::ActiveModel {
        fn from(domain: DomainBudget) -> Self {
            use sea_orm::{ActiveValue, Set};

            Self {
                id: if domain.id > 0 {
                    ActiveValue::Set(domain.id)
                } else {
                    ActiveValue::NotSet
                },
                user_id: Set(domain.user_id),
                category_id: Set(domain.category_id),
                amount_limit: Set(domain.amount_limit),
                month: Set(domain.month),
                current_spent: Set(domain.current_spent),
                updated_at: Set(chrono::Utc::now().naive_utc()),
            }
        }
    }
}

pub mod category {
    use crate::models::{
        domain::category::{Category as DomainCategory, CategoryType as DomainCatType},
        entities::category::{CategoryType as DbCatType, Model as CategoryEntity},
    };

    pub trait CategoryDomainExt {
        fn into_domain(self) -> DomainCategory;
    }

    impl CategoryDomainExt for CategoryEntity {
        fn into_domain(self) -> DomainCategory {
            DomainCategory {
                id: self.id,
                user_id: self.user_id,
                name: self.name,
                category_type: match self.category_type {
                    DbCatType::Income => DomainCatType::Income,
                    DbCatType::Expense => DomainCatType::Expense,
                },
            }
        }
    }

    impl From<DomainCategory> for crate::models::entities::category::ActiveModel {
        fn from(domain: DomainCategory) -> Self {
            use sea_orm::{ActiveValue, Set};

            Self {
                id: if domain.id > 0 {
                    ActiveValue::Set(domain.id)
                } else {
                    ActiveValue::NotSet
                },
                user_id: Set(domain.user_id),
                name: Set(domain.name),
                category_type: Set(match domain.category_type {
                    DomainCatType::Income => DbCatType::Income,
                    DomainCatType::Expense => DbCatType::Expense,
                }),
            }
        }
    }

    impl From<DbCatType> for DomainCatType {
        fn from(db: DbCatType) -> Self {
            match db {
                DbCatType::Income => DomainCatType::Income,
                DbCatType::Expense => DomainCatType::Expense,
            }
        }
    }

    impl From<DomainCatType> for DbCatType {
        fn from(domain: DomainCatType) -> Self {
            match domain {
                DomainCatType::Income => DbCatType::Income,
                DomainCatType::Expense => DbCatType::Expense,
            }
        }
    }
}

pub mod user {
    use crate::models::{domain::user::User as DomainUser, entities::user::Model as UserEntity};

    pub trait UserDomainExt {
        fn into_domain(self) -> DomainUser;
    }

    impl UserDomainExt for UserEntity {
        fn into_domain(self) -> DomainUser {
            DomainUser {
                id: self.id,
                email: self.email,
                password_hash: self.password_hash,
                created_at: self.created_at,
            }
        }
    }

    impl From<DomainUser> for crate::models::entities::user::ActiveModel {
        fn from(domain: DomainUser) -> Self {
            use sea_orm::{ActiveValue, Set};

            Self {
                id: if domain.id > 0 {
                    ActiveValue::Set(domain.id)
                } else {
                    ActiveValue::NotSet
                },
                email: Set(domain.email),
                password_hash: Set(domain.password_hash),
                created_at: Set(domain.created_at),
            }
        }
    }
}
