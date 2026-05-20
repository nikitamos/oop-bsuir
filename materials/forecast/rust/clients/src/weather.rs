use std::error::Error;

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

#[cfg(false)]
pub mod fake {
    use anyhow::anyhow;
    use anyhow::bail;
    use chrono::NaiveDate;
    use reqwest::StatusCode;

    use crate::BadStatusError;

    use models::weather::Forecast;

    use models::weather::LocationSpec;

    use std::error::Error;

    use rust_decimal::Decimal;

    use super::WeatherDataClient;

    #[derive(Clone)]
    pub struct FakeSuccessWeatherDataClient;

    impl WeatherDataClient for FakeSuccessWeatherDataClient {
        async fn location_current_temperature(
            &self,
            lat: Decimal,
            lon: Decimal,
        ) -> anyhow::Result<Decimal> {
            Ok(Decimal::ZERO)
        }

        async fn current_temp(&self, locations: &[LocationSpec]) -> anyhow::Result<Vec<Decimal>> {
            Ok(vec![Decimal::ZERO; locations.len()])
        }

        async fn forecast(
            &self,
            locations: &[LocationSpec],
            date: NaiveDate,
        ) -> anyhow::Result<Vec<Forecast>> {
            Ok(vec![Default::default(); locations.len()])
        }
    }

    #[derive(Clone)]
    pub struct FakeErrorWeatherDataClient;

    impl WeatherDataClient for FakeErrorWeatherDataClient {
        async fn location_current_temperature(
            &self,
            lat: Decimal,
            lon: Decimal,
        ) -> anyhow::Result<Decimal> {
            bail!(Self::error())
        }

        async fn current_temp(&self, locations: &[LocationSpec]) -> anyhow::Result<Vec<Decimal>> {
            Self::error()
        }

        async fn forecast(
            &self,
            locations: &[LocationSpec],
            date: NaiveDate,
        ) -> anyhow::Result<Vec<Forecast>> {
            Self::error()
        }
    }

    impl FakeErrorWeatherDataClient {
        pub(crate) fn error<T>() -> anyhow::Result<T> {
            Err(Box::new(BadStatusError::new(
                "fake weather client",
                StatusCode::NOT_ACCEPTABLE,
            )))
        }
    }
}
