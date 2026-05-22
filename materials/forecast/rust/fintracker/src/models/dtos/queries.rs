use chrono::NaiveDate;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct MonthQuery {
    pub month: NaiveDate,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct MonthCountQuery {
    #[param(example = "6")]
    pub months: Option<u32>,
}
