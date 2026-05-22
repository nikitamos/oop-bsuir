use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct MonthQuery {
    #[param(example = "2025-06-01")]
    pub month: String,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct MonthCountQuery {
    #[param(example = "6")]
    pub months: Option<u32>,
}
