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

        let urls = build_urls(&self.base_url, "weather", &cities, &coords, &self.api_key);
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

        let urls = build_urls(&self.base_url, "forecast", &cities, &coords, &self.api_key);
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
            .filter_map(|i| i.rain.as_ref().map_or(Some(0.0), |r| r.hours_3))
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
    endpoint: &str,
    cities: &[String],
    coords: &[(Decimal, Decimal)],
    api_key: &str,
) -> Vec<String> {
    cities
        .iter()
        .map(|c| format!("{base}/{endpoint}?q={c}&appid={api_key}&units=metric"))
        .chain(coords.iter().map(|(lat, lon)| {
            format!("{base}/{endpoint}?lat={lat}&lon={lon}&appid={api_key}&units=metric")
        }))
        .collect()
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use crate::{LocationSpec, WeatherDataClient};

    use super::OpenWeatherClient;
    use anyhow::Result;
    use chrono::NaiveDate;
    use rust_decimal::{
        Decimal,
        prelude::{FromPrimitive, ToPrimitive},
    };
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path, query_param},
    };
    #[tokio::test]
    async fn test_current_temp_multiple_locations() -> Result<()> {
        let mock = MockServer::start().await;

        // Mock for both coordinate requests
        Mock::given(method("GET"))
            .and(path("/weather"))
            .and(query_param("appid", "key"))
            .and(query_param("units", "metric"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "main": { "temp": 18.0 }
            })))
            .expect(2)
            .mount(&mock)
            .await;

        let client = OpenWeatherClient::new("key".into(), mock.uri());
        let locations = vec![
            LocationSpec::Coordinates {
                lat: Decimal::from(40),
                lon: Decimal::from(-74),
            },
            LocationSpec::Coordinates {
                lat: Decimal::from(51),
                lon: Decimal::from(-0),
            },
        ];

        let temps = client.current_temp(&locations).await?;
        assert_eq!(temps, vec![Decimal::from_f64(18.0).unwrap(); 2]);
        Ok(())
    }

    #[tokio::test]
    async fn test_forecast_aggregates_daily_data() -> Result<()> {
        let mock = MockServer::start().await;
        let target_date = NaiveDate::from_ymd_opt(2025, 2, 10).unwrap();

        // Mock forecast response with 3-hour steps for the target day
        Mock::given(method("GET"))
        .and(path("/forecast"))
        .and(query_param("lat", "40"))
        .and(query_param("lon", "-74"))
        .and(query_param("appid", "key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "list": [
                { "dt": 1739203200, "main": { "temp": 12.0, "humidity": 60 }, "wind": { "deg": 270 }, "pop": 0.1 },
                { "dt": 1739214000, "main": { "temp": 14.0, "humidity": 50 }, "wind": { "deg": 280 }, "pop": 0.2 },
                { "dt": 1739224800, "main": { "temp": 16.0, "humidity": 40 }, "wind": { "deg": 290 }, "rain": { "3h": 0.5 } }
            ]
        })))
        .mount(&mock)
        .await;

        let client = OpenWeatherClient::new("key".into(), mock.uri());
        let forecast = client
            .forecast(
                &[LocationSpec::Coordinates {
                    lat: Decimal::from(40),
                    lon: Decimal::from(-74),
                }],
                target_date,
            )
            .await?;

        assert_eq!(forecast.len(), 1);
        // Avg temp: (12+14+16)/3 = 14.0
        assert!((forecast[0].temperature.to_f64().unwrap() - 14.0).abs() < 0.01);
        // Avg humidity: (60+50+40)/3 = 50
        assert_eq!(forecast[0].humidity, 50);
        // Precip: prefers rain.3h (0.5) over pop
        assert_eq!(forecast[0].precipitation, Decimal::from_f64(0.5).unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_handles_api_error() {
        let mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/weather"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "cod": "404", "message": "city not found"
            })))
            .mount(&mock)
            .await;

        let client = OpenWeatherClient::new("key".into(), mock.uri());
        let result = client
            .location_current_temperature(Decimal::from(999), Decimal::from(999))
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));
    }

    // Add to tests/openweather_client.rs

    #[tokio::test]
    async fn test_current_temp_with_city_name() -> Result<()> {
        let mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/weather"))
            .and(query_param("q", "London"))
            .and(query_param("appid", "key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "main": { "temp": 15.5 }
            })))
            .mount(&mock)
            .await;

        let client = OpenWeatherClient::new("key".into(), mock.uri());
        let locations = vec![LocationSpec::City {
            city: "London".into(),
        }];

        let temps = client.current_temp(&locations).await?;
        assert_eq!(temps, vec![Decimal::from_f64(15.5).unwrap()]);
        Ok(())
    }

    #[tokio::test]
    async fn test_forecast_with_city_name() -> Result<()> {
        let mock = MockServer::start().await;
        let target_date = NaiveDate::from_ymd_opt(2025, 2, 10).unwrap();

        Mock::given(method("GET"))
            .and(path("/forecast"))
            .and(query_param("q", "Zhabinka"))
            .and(query_param("appid", "key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "list": [{
                    "dt": 1739224800,
                    "main": { "temp": 10.0, "humidity": 70 },
                    "wind": { "deg": 180 },
                    "pop": 0.3
                }]
            })))
            .mount(&mock)
            .await;

        let client = OpenWeatherClient::new("key".into(), mock.uri());
        let locations = vec![LocationSpec::City {
            city: "Zhabinka".into(),
        }];

        let forecast = client.forecast(&locations, target_date).await?;
        assert_eq!(forecast.len(), 1);
        assert_eq!(forecast[0].temperature, Decimal::from_f64(10.0).unwrap());
        assert_eq!(forecast[0].humidity, 70);
        assert_eq!(forecast[0].wind_direction, "S"); // 180° = South
        Ok(())
    }
}
