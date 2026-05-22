pub struct ExpensesSummary {}
pub struct MonthlySummary {}

pub mod account;
pub mod auth;
pub mod budget;
pub mod transaction;

pub mod category;

pub mod analytics;
pub mod queries;

use serde::Serialize;
use std::fmt::Debug;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
