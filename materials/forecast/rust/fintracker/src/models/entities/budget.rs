use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "budgets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub user_id: i64,
    pub category_id: i64,
    #[sea_orm(column_type = "Decimal(Some((16, 2)))")]
    pub amount_limit: Decimal,
    #[sea_orm(column_type = "Date")]
    pub month: chrono::NaiveDate,
    #[sea_orm(column_type = "Decimal(Some((16, 2)))", default_value = "0")]
    pub current_spent: Decimal,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: chrono::NaiveDateTime,
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
    
    #[sea_orm(
        belongs_to = "super::category::Entity",
        from = "Column::CategoryId",
        to = "super::category::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Category,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
