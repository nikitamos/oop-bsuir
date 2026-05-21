use anyhow::bail;
use chrono::{NaiveDate, Utc};
use futures::future::join_all;
use models::weather::{
    Coordinates, Forecast, LocationSpec,
    google::{CurrentConditions, GoogleForecastResponse},
};
use rust_decimal::Decimal;

use crate::{LocationNormalizer, WeatherDataClient};

pub struct GoogleWeatherClient {
    base_url: String,
    api_key: String,
    normalizer: LocationNormalizer,
}

impl GoogleWeatherClient {
    pub fn from_env() -> Self {
        Self {
            base_url: shared::utils::getenv("GOOGLE_API_KEY", "xxx".to_string()),
            api_key: shared::utils::getenv("GOOGLE_BASE_URL", "https://weather.googleapis.com/v1/".to_string()),
            normalizer: LocationNormalizer::default(),
        }
    }
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            normalizer: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl WeatherDataClient for GoogleWeatherClient {
    async fn location_current_temperature(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> anyhow::Result<Decimal> {
        self.current_temp(&[LocationSpec::Coordinates { lat, lon }])
            .await
            .map(|mut v| v.remove(0))
    }

    async fn current_temp(&self, locations: &[LocationSpec]) -> anyhow::Result<Vec<Decimal>> {
        let locations: Vec<_> = self.normalizer.normalize_locations(locations).collect();
        for loc in &locations {
            if loc.is_none() {
                bail!("Given city is unsupported by the backend")
            }
        }
        let futures =
            locations
                .into_iter()
                .filter_map(|x| x)
                .map(|Coordinates { lat, lon }| async move {
                    self.fetch_location_conditions(lat, lon).await
                });

        join_all(futures).await.into_iter().collect()
    }

    async fn forecast(
        &self,
        locations: &[LocationSpec],
        date: chrono::prelude::NaiveDate,
    ) -> anyhow::Result<Vec<models::weather::Forecast>> {
        let locations: Vec<_> = self.normalizer.normalize_locations(locations).collect();
        for loc in &locations {
            if loc.is_none() {
                bail!("Given city is unsupported by backend")
            }
        }
        let futures =
            locations
                .into_iter()
                .filter_map(|x| x)
                .map(|Coordinates { lat, lon }| async move {
                    self.fetch_forecast(lat, lon, date).await
                });

        join_all(futures).await.into_iter().collect()
    }
}

impl GoogleWeatherClient {
    async fn fetch_location_conditions(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> anyhow::Result<Decimal> {
        let url = format!(
            "{}currentConditions:lookup?key={}&location.latitude={}&location.longitude={}",
            self.base_url, self.api_key, lat, lon
        );
        let resp = reqwest::get(url).await?;
        let conditions: CurrentConditions = serde_json::de::from_str(&resp.text().await?)?;
        Ok(conditions.temperature.degrees)
    }
    async fn fetch_forecast(
        &self,
        lat: Decimal,
        lon: Decimal,
        date: NaiveDate,
    ) -> anyhow::Result<models::weather::Forecast> {
        let today = Utc::now().date_naive();
        let days = (date - today).num_days();
        if days < 0 || days > 10 {
            bail!("Date is outside of possible range");
        }
        let day = days as usize;
        let url = format!(
            "{}forecast/days:lookup?key={}&location.latitude={lat}&location.longitude={lon}",
            self.base_url, self.api_key
        );
        let resp = reqwest::get(url).await?;
        let mut body: GoogleForecastResponse = serde_json::de::from_str(&resp.text().await?)?;
        let forecast = body.forecast_days.remove(day);
        Ok(Forecast {
            temperature: forecast.max_temperature.degrees,
            wind_direction: crate::deg_to_cardinal(
                forecast.daytime_forecast.wind.direction.degrees.unwrap(),
            ),
            humidity: forecast.daytime_forecast.relative_humidity,
            precipitation: forecast
                .daytime_forecast
                .precipitation
                .qpf
                .map_or(Decimal::ZERO, |p| p.quantity),
        })
    }
}
