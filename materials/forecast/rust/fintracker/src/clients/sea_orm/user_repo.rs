use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, Set};

use crate::{
    models::{
        domain::user::User as DomainUser,
        entities::user::{self, Entity as UserEntity},
    },
    shared::error::Result,
};

use super::{conversion::user::UserDomainExt, error::DbResultExt};

pub struct SeaOrmUserRepository {
    db: sea_orm::DatabaseConnection,
}

impl SeaOrmUserRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl super::UserRepository for SeaOrmUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<DomainUser>> {
        let entity = UserEntity::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_db_err()?;

        Ok(entity.map(|e| e.into_domain()))
    }

    async fn create(&self, user: &DomainUser) -> Result<DomainUser> {
        let active_model: user::ActiveModel = user.clone().into();

        let inserted = active_model.insert(&self.db).await.map_db_err()?;

        Ok(inserted.into_domain())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<DomainUser>> {
        let entity = UserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_db_err()?;

        Ok(entity.map(|e| e.into_domain()))
    }
}
