use anyhow::bail;
use chrono::{NaiveDate, Utc};
use futures::future::join_all;
use models::weather::{
    Coordinates, Forecast, LocationSpec,
    google::{CurrentConditions, GoogleForecastResponse},
};
use rust_decimal::Decimal;

use crate::{DateProvider, DefaultDateProvider, LocationNormalizer, WeatherDataClient};

pub struct GoogleWeatherClient<DP: DateProvider = DefaultDateProvider> {
    base_url: String,
    api_key: String,
    normalizer: LocationNormalizer,
    date_provider: DP,
}

impl GoogleWeatherClient {
    pub fn from_env() -> Self {
        Self {
            base_url: shared::utils::getenv(
                "GOOGLE_BASE_URL",
                "https://weather.googleapis.com/v1".to_string(),
            ),
            api_key: shared::utils::getenv("GOOGLE_API_KEY", "xxx".to_string()),
            normalizer: LocationNormalizer::default(),
            date_provider: DefaultDateProvider::default(),
        }
    }
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            normalizer: Default::default(),
            date_provider: DefaultDateProvider::default(),
        }
    }
}

impl<DP: DateProvider> GoogleWeatherClient<DP> {
    pub fn new_with_date(base_url: String, api_key: String, date_provider: DP) -> Self {
        Self {
            base_url,
            api_key,
            normalizer: Default::default(),
            date_provider,
        }
    }
}

#[async_trait::async_trait]
impl<DP: DateProvider> WeatherDataClient for GoogleWeatherClient<DP> {
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

impl<DP: DateProvider> GoogleWeatherClient<DP> {
    async fn fetch_location_conditions(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> anyhow::Result<Decimal> {
        let url = format!(
            "{}/currentConditions:lookup?key={}&location.latitude={}&location.longitude={}",
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
        let today = self.date_provider.today();
        let days = (date - today).num_days();
        if days < 0 || days > 10 {
            bail!("Date is outside of possible range");
        }
        let day = days as usize;
        let url = format!(
            "{}/forecast/days:lookup?key={}&location.latitude={lat}&location.longitude={lon}",
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

#[cfg(test)]
#[coverage(off)]
mod test {
    struct FakeDateProvider {
        date: NaiveDate,
    }
    impl DateProvider for FakeDateProvider {
        fn today(&self) -> NaiveDate {
            self.date
        }
    }

    const CURRENT_CONDITIONS: &str = "currentConditions:lookup";
    const FORECAST: &str = "forecast/days:lookup";

    use crate::{DateProvider, WeatherDataClient};

    use super::{GoogleWeatherClient, LocationSpec};
    use anyhow::Result;
    use chrono::NaiveDate;
    use models::weather::google::{
        ConditionForecast, CurrentConditions, ForecastDay, GoogleForecastResponse, Precipitation,
        QuantitativePrecipitationForecast, Temperature, WindDirection,
    };
    use rstest::rstest;
    use rust_decimal::Decimal;
    use wiremock::{
        Mock, MockBuilder, MockServer, ResponseTemplate,
        matchers::{method, path, path_regex, query_param},
    };

    pub fn expect_request(endpoint: &str, key: &str, lat: &str, lon: &str) -> MockBuilder {
        Mock::given(method("GET"))
            .and(path(endpoint))
            .and(query_param("key", key))
            .and(query_param("location.latitude", lat))
            .and(query_param("location.longitude", lon))
    }

    #[rstest]
    #[tokio::test]
    async fn location_current_temperature() -> Result<()> {
        let mock = MockServer::start().await;

        expect_request("currentConditions:lookup", "test-key", "40", "-74")
            .respond_with(
                ResponseTemplate::new(200).set_body_json(CurrentConditions::with_temp(
                    Decimal::from_str_exact("22.5").unwrap(),
                )),
            )
            .mount(&mock)
            .await;

        let client = GoogleWeatherClient::new(mock.uri(), "test-key".into());
        let temp = client
            .current_temp(&[LocationSpec::from_coord_str("40", "-74").unwrap()])
            .await?;

        assert_eq!(temp[0], Decimal::from_str_exact("22.5").unwrap());
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn current_temp_multiple_locations() -> Result<()> {
        let mock = MockServer::start().await;
        let expected_temps = vec![Decimal::from(18), Decimal::from(-4)];

        expect_request(CURRENT_CONDITIONS, "test-key", "40", "-74")
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(CurrentConditions::with_temp(expected_temps[0])),
            )
            .mount(&mock)
            .await;
        expect_request(CURRENT_CONDITIONS, "test-key", "51", "0")
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(CurrentConditions::with_temp(expected_temps[1])),
            )
            .mount(&mock)
            .await;

        let client = GoogleWeatherClient::new(mock.uri(), "test-key".into());
        let locations = vec![
            LocationSpec::from_coord_str("40", "-74").unwrap(),
            LocationSpec::from_coord_str("51", "0").unwrap(),
        ];

        let temps = client.current_temp(&locations).await?;
        assert_eq!(expected_temps, temps);
        Ok(())
    }

    #[rstest]
    #[case::feb_10("2026-05-23", Decimal::from_str_exact("13.3").unwrap(), Decimal::from_str_exact("12.0").unwrap())]
    #[case::feb_11("2026-05-26", Decimal::from_str_exact("12.0").unwrap(), Decimal::from_str_exact("19.0").unwrap())]
    #[tokio::test]
    async fn forecast(
        #[case] date_str: &str,
        #[case] min_temp: Decimal,
        #[case] max_temp: Decimal,
    ) -> Result<()> {
        use models::weather::google::Wind;

        let target_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let expected_temp = (min_temp + max_temp) / Decimal::from(2);
        let mock = MockServer::start().await;
        let date = FakeDateProvider {
            date: NaiveDate::from_ymd_opt(2026, 05, 21).unwrap(),
        };
        let offset = (target_date - date.date).num_days() as usize;

        let mut gibberish = GoogleForecastResponse {
            forecast_days: vec![
                ForecastDay {
                    daytime_forecast: ConditionForecast {
                        relative_humidity: 100,
                        precipitation: Precipitation {
                            qpf: Some(QuantitativePrecipitationForecast {
                                quantity: Decimal::from(12),
                                unit: "mm".to_string()
                            })
                        },
                        wind: Wind {
                            direction: WindDirection {
                                degrees: Some(180.0),
                                cardinal: None
                            }
                        }
                    },
                    max_temperature: Temperature {
                        degrees: Decimal::ZERO
                    },
                    min_temperature: Temperature {
                        degrees: Decimal::ZERO
                    }
                };
                10
            ],
        };
        gibberish.forecast_days[offset].max_temperature.degrees = max_temp;
        gibberish.forecast_days[offset].min_temperature.degrees = max_temp;
        gibberish.forecast_days[offset]
            .daytime_forecast
            .relative_humidity = 54;
        gibberish.forecast_days[offset]
            .daytime_forecast
            .wind
            .direction
            .degrees = Some(273.0);

        expect_request(FORECAST, "key", "40", "-74")
            .respond_with(ResponseTemplate::new(200).set_body_json(gibberish))
            .mount(&mock)
            .await;

        let client = GoogleWeatherClient::new_with_date(mock.uri(), "key".into(), date);
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
        assert!(
            (forecast[0].temperature - expected_temp).abs()
                < Decimal::from_f32_retain(0.01).unwrap()
        );
        assert_eq!(forecast[0].humidity, 54);
        assert_eq!(forecast[0].wind_direction, "W");
        Ok(())
    }

    #[tokio::test]
    async fn current_temp_api_error() {
        let mock = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(format!(r"^{CURRENT_CONDITIONS}.*")))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "error": { "message": "Location not found" }
            })))
            .mount(&mock)
            .await;

        let client = GoogleWeatherClient::new(mock.uri(), "key".into());
        let result = client
            .location_current_temperature(Decimal::from(999), Decimal::from(999))
            .await;

        assert!(result.is_err());
    }
}
