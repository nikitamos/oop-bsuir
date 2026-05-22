use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,    
    #[sea_orm(unique, indexed)]
    pub email: String, 
    pub password_hash: String, 
    #[sea_orm(column_type = "Timestamp")]
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::account::Entity")]
    Account,
    
    #[sea_orm(has_many = "super::category::Entity")]
    Category,
    
    #[sea_orm(has_many = "super::budget::Entity")]
    Budget,
}

impl ActiveModelBehavior for ActiveModel {}