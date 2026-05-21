use std::str::FromStr;

use anyhow::anyhow;
use chrono::NaiveDate;
use futures::future::join_all;
use models::weather::{owm::*, segregate_locations};
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
    pub fn from_env() -> OpenWeatherClient {
        OpenWeatherClient::new(
            shared::utils::getenv("OPENWEATHER_API_KEY", String::new()),
            shared::utils::getenv("OPENWEATHER_BASE_URL", String::new()),
        )
    }
}

#[async_trait::async_trait]
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
        let (cities, coords) = segregate_locations(locations);
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();

        let urls = build_urls(&base_url, &cities, &coords, &api_key);
        let futures = urls
            .into_iter()
            .map(|url| async move { Self::fetch_temperature(&url).await });

        join_all(futures).await.into_iter().collect()
    }

    async fn forecast(
        &self,
        locations: &[models::weather::LocationSpec],
        date: NaiveDate,
    ) -> anyhow::Result<Vec<models::weather::Forecast>> {
        let (cities, coords) = segregate_locations(locations);
        // OpenWeather uses `/forecast` instead of `/weather`
        let forecast_base = self.base_url.replace("/weather", "/forecast");
        let api_key = self.api_key.clone();

        let urls = build_urls(&forecast_base, &cities, &coords, &api_key);
        let futures = urls.into_iter().map(|url| {
            let target = date;
            async move { Self::fetch_forecast(&url, target).await }
        });

        join_all(futures).await.into_iter().collect()
    }
}

impl OpenWeatherClient {
    async fn fetch_temperature(url: &str) -> anyhow::Result<Decimal> {
        let resp = reqwest::get(url).await?;
        if resp.status() != StatusCode::OK {
            return Err(anyhow!(BadStatusError::new("OpenWeather", resp.status())));
        }
        let data: OpenWeatherResponse = serde_json::de::from_str(&resp.text().await?)?;
        Ok(data.main.temp)
    }

    async fn fetch_forecast(
        url: &str,
        target_date: NaiveDate,
    ) -> anyhow::Result<models::weather::Forecast> {
        let resp = reqwest::get(url).await?;
        if resp.status() != StatusCode::OK {
            return Err(anyhow!(BadStatusError::new("OpenWeather", resp.status())));
        }

        let data: OpenWeatherForecastResponse = serde_json::de::from_str(&resp.text().await?)?;

        // Filter 3-hour steps to the requested day
        let day_items: Vec<&ForecastItem> = data
            .list
            .iter()
            .filter(|item| {
                chrono::DateTime::from_timestamp(item.dt, 0)
                    .map(|dt| dt.date_naive() == target_date)
                    .unwrap_or(false)
            })
            .collect();

        if day_items.is_empty() {
            return Err(anyhow!("No forecast data available for {}", target_date));
        }

        let count = Decimal::from(day_items.len() as i64);

        // Aggregate daily metrics
        let temperature: Decimal = day_items
            .iter()
            .filter_map(|i| Decimal::from_f64_retain(i.main.temp))
            .sum::<Decimal>()
            / count;

        let humidity: u32 =
            day_items.iter().map(|i| i.main.humidity).sum::<u32>() / day_items.len() as u32;

        let wind_deg: f64 = {
            let sum: f64 = day_items.iter().filter_map(|i| i.wind.deg).sum();
            let valid = day_items.iter().filter(|i| i.wind.deg.is_some()).count();
            if valid > 0 { sum / valid as f64 } else { 0.0 }
        };

        let precipitation: Decimal = day_items
            .iter()
            .filter_map(|i| i.rain.as_ref().and_then(|r| r.hours_3).or(i.pop))
            .filter_map(|v| Decimal::from_str(&format!("{}", v)).ok())
            .sum::<Decimal>();

        Ok(models::weather::Forecast {
            temperature,
            wind_direction: crate::deg_to_cardinal(wind_deg),
            humidity,
            precipitation,
        })
    }
}

// Helper to cleanly merge city and coordinate URLs
fn build_urls(
    base: &str,
    cities: &[String],
    coords: &[(Decimal, Decimal)],
    api_key: &str,
) -> Vec<String> {
    cities
        .iter()
        .map(|c| format!("{}?q={}&appid={}&units=metric", base, c, api_key))
        .chain(coords.iter().map(|(lat, lon)| {
            format!(
                "{}?lat={}&lon={}&appid={}&units=metric",
                base, lat, lon, api_key
            )
        }))
        .collect()
}
