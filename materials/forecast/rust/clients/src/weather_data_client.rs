use chrono::prelude::*;
use models::weather::{Forecast, LocationSpec};
use rust_decimal::Decimal;


#[allow(async_fn_in_trait)]
pub trait WeatherDataClient {
    fn location_current_temperature(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> impl Future<Output = anyhow::Result<Decimal>> + Send;
    fn current_temp(
        &self,
        locations: &[LocationSpec],
    ) -> impl Future<Output = anyhow::Result<Vec<Decimal>>> + Send;
    fn forecast(
        &self,
        locations: &[LocationSpec],
        date: NaiveDate,
    ) -> impl Future<Output = anyhow::Result<Vec<Forecast>>> + Send;
}
