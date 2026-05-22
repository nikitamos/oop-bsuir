use sea_orm::entity::prelude::*;

/// Тип категории совпадает с типом транзакции
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(10))")]
pub enum CategoryType {
    #[sea_orm(string_value = "income")]
    Income,
    #[sea_orm(string_value = "expense")]
    Expense,
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub user_id: i64,
    #[sea_orm(column_type = "String(StringLen::N(100))", indexed)]
    pub name: String,
    #[sea_orm(enum_name = "CategoryType")]
    pub category_type: CategoryType,
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
    #[sea_orm(has_many = "super::budget::Entity")]
    Budget,
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

impl Related<super::budget::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Budget.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}