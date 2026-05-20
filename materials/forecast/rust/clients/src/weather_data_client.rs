use std::error::Error;

use chrono::prelude::*;
use models::weather::{Forecast, LocationSpec};
use reqwest::StatusCode;
use rust_decimal::Decimal;

use crate::BadStatusError;

#[allow(async_fn_in_trait)]
pub trait WeatherDataClient {
    async fn location_current_temperature(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> anyhow::Result<Decimal>;
    async fn current_temp(&self, locations: &[LocationSpec]) -> anyhow::Result<Vec<Decimal>>;
    async fn forecast(
        &self,
        locations: &[LocationSpec],
        date: NaiveDate,
    ) -> anyhow::Result<Vec<Forecast>>;
}
