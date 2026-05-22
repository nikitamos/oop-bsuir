use serde::Serialize;
use utoipa::ToSchema;

use crate::models::domain::category::CategoryType;

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub category_type: CategoryTypeDto,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all="snake_case")]
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
