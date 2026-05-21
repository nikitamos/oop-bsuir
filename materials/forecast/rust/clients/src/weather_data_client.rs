use chrono::prelude::*;
use models::weather::{Forecast, LocationSpec};
use rust_decimal::Decimal;

#[async_trait::async_trait]
pub trait WeatherDataClient: Send + Sync {
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
