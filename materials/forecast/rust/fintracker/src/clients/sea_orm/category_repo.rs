use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait};

use crate::{
    models::{
        domain::category::Category as DomainCategory,
        entities::category::{ActiveModel, Entity as CategoryEntity},
    },
    shared::error::Result,
};

use super::{conversion::category::CategoryDomainExt, error::DbResultExt};

pub struct SeaOrmCategoryRepository {
    db: sea_orm::DatabaseConnection,
}

impl SeaOrmCategoryRepository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl super::CategoryRepository for SeaOrmCategoryRepository {
    async fn find_all(&self) -> Result<Vec<DomainCategory>> {
        let entities = CategoryEntity::find().all(&self.db).await.map_db_err()?;

        Ok(entities.into_iter().map(|e| e.into_domain()).collect())
    }

    async fn find_by_id(&self, id: i64) -> Result<DomainCategory> {
        let entity = CategoryEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_db_err()?
            .ok_or(crate::shared::error::ApplicationError::NotFound(format!(
                "Category with id {id} was not found"
            )))?;

        Ok(entity.into_domain())
    }

    async fn create(&self, category: DomainCategory) -> Result<i64> {
        let model = ActiveModel {
            id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(category.user_id),
            name: ActiveValue::Set(category.name),
            category_type: ActiveValue::Set(category.category_type.into()),
        };
        let saved = model.insert(&self.db).await.map_db_err()?;
        Ok(saved.id)
    }
}
