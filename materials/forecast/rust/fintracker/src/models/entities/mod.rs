pub mod prelude {
    pub use super::{
        user::Entity as User,
        account::Entity as Account,
        category::Entity as Category,
        transaction::Entity as Transaction,
        budget::Entity as Budget,
    };
}

pub mod user;
pub mod account;
pub mod category;
pub mod transaction;
pub mod budget;

pub use account::{AccountType as DbAccountType, Model as AccountModel};
pub use category::{CategoryType as DbCategoryType, Model as CategoryModel};
pub use transaction::{TransactionType as DbTransactionType, Model as TransactionModel};
pub use budget::Model as BudgetModel;
pub use user::Model as UserModel;