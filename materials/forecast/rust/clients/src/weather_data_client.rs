use std::error::Error;

use chrono::prelude::*;
use models::weather::{Forecast, LocationSpec};
use rust_decimal::Decimal;

#[allow(async_fn_in_trait)]
pub trait WeatherDataClient {
    async fn location_current_temperature(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> Result<Decimal, Box<dyn std::error::Error>>;
    async fn current_temp(
        &self,
        locations: &[LocationSpec],
    ) -> Result<Vec<Decimal>, Box<dyn Error>> {
        todo!()
    }
    async fn forecast(
        &self,
        locations: &[LocationSpec],
        date: NaiveDate,
    ) -> Result<Vec<Forecast>, Box<dyn Error>> {
        todo!()
    }
}
