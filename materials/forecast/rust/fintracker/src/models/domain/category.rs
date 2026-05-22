use crate::models::domain::transaction::TransactionType;

pub struct Category {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub category_type: CategoryType,
}

pub type CategoryType = TransactionType;
