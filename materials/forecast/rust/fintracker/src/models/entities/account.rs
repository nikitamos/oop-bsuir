use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum AccountType {
    #[sea_orm(string_value = "cash")]
    Cash,
    #[sea_orm(string_value = "bank_card")]
    BankCard,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub user_id: i64,
    #[sea_orm(column_type = "String(StringLen::N(100))")]
    pub name: String,
    #[sea_orm(enum_name = "AccountType")]
    pub account_type: AccountType,
    #[sea_orm(column_type = "Decimal(Some((16, 2)))")]
    pub balance: Decimal,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transaction,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
