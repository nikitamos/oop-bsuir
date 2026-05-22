use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

/// Тип транзакции — дублирует логику домена, но для БД
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
pub enum TransactionType {
    #[sea_orm(string_value = "income")]
    Income,
    #[sea_orm(string_value = "expense")]
    Expense,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    
    pub account_id: i64,
    
    pub category_id: i64,
    
    /// Сумма в минимальных единицах валюты
    #[sea_orm(column_type = "Decimal(Some((19, 2)))")]
    pub amount: Decimal,
    
    #[sea_orm(enum_name = "TransactionType")]
    pub transaction_type: TransactionType,
    
    #[sea_orm(column_type = "String(StringLen::N(500))", nullable)]
    pub description: Option<String>,
    
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub transaction_date: chrono::NaiveDateTime,
    
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Account,
    
    #[sea_orm(
        belongs_to = "super::category::Entity",
        from = "Column::CategoryId",
        to = "super::category::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Category,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}