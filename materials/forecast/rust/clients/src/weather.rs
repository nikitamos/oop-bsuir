use anyhow::anyhow;
use reqwest::StatusCode;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{BadStatusError, WeatherDataClient};

#[derive(Deserialize)]
struct Main {
    pub temp: Decimal,
}

#[derive(Deserialize)]
struct OpenWeatherResponse {
    pub main: Main,
}

#[derive(Clone)]
pub struct OpenWeatherClient {
    api_key: String,
    base_url: String,
}

impl OpenWeatherClient {
    pub fn new(api_key: String, base_url: String) -> OpenWeatherClient {
        OpenWeatherClient { api_key, base_url }
    }
}

impl WeatherDataClient for OpenWeatherClient {
    async fn location_current_temperature(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> anyhow::Result<Decimal> {
        let url = format!(
            "{}?lat={}&lon={}&appid={}&units=metric",
            self.base_url, lat, lon, self.api_key
        );
        let resp = reqwest::get(url).await?;
        if resp.status() != StatusCode::OK {
            return Err(anyhow!(BadStatusError::new("OpenWeather", resp.status())));
        }
        let data: OpenWeatherResponse = serde_json::from_str(&resp.text().await?)?;
        Ok(data.main.temp)
    }

    async fn current_temp(
        &self,
        locations: &[models::weather::LocationSpec],
    ) -> anyhow::Result<Vec<Decimal>> {
        Err(anyhow::anyhow!("not implemented"))
    }

    async fn forecast(
        &self,
        locations: &[models::weather::LocationSpec],
        date: chrono::prelude::NaiveDate,
    ) -> anyhow::Result<Vec<models::weather::Forecast>> {
        Err(anyhow::anyhow!("not implemented"))
    }
}
