use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::domain::category::{Category, CategoryType};

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub category_type: CategoryTypeDto,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CategoryCreateRequest {
    #[schema(example = "Traveling")]
    pub name: String,
    #[schema(example = CategoryTypeDto::Expense)]
    pub category_type: CategoryTypeDto,
}

impl From<Category> for CategoryResponse {
    fn from(value: Category) -> Self {
        Self {
            id: value.id,
            name: value.name,
            category_type: value.category_type.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CategoryTypeDto {
    Expense,
    Income,
}

impl From<CategoryTypeDto> for CategoryType {
    fn from(value: CategoryTypeDto) -> Self {
        match value {
            CategoryTypeDto::Expense => Self::Expense,
            CategoryTypeDto::Income => Self::Income,
        }
    }
}

impl From<CategoryType> for CategoryTypeDto {
    fn from(value: CategoryType) -> Self {
        match value {
            CategoryType::Expense => Self::Expense,
            CategoryType::Income => Self::Income,
        }
    }
}
