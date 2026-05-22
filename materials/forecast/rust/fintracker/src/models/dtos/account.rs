use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAccountRequest {
    #[schema(example = "Наличные")]
    pub name: String,
    #[schema(example = "Cash")]
    pub account_type: String,
    pub initial_balance: Decimal,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AccountResponse {
    pub id: i64,
    pub name: String,
    #[schema(example = "Cash")]
    pub account_type: String,
    #[schema(example = "137.42")]
    pub balance: Decimal,
}
