// tests/weather_handlers_test.rs
use axum::http::StatusCode;
use axum_test::TestServer;
use mockall::predicate::*;
use mockall::*;
use rstest::*;
use rust_decimal::Decimal;
use serde_json::json;

use crate::WeatherHandler;
use chrono::NaiveDate;
use clients::WeatherDataClient;
use models::weather::{CurrentWeather, Forecast, LocationSpec};
use shared::responses::{StatusResponse, SuccessResponse};

// Mock the WeatherDataClient
mock! {
    pub WeatherClient {}
    impl WeatherDataClient for WeatherClient {
        async fn location_current_temperature(
            &self,
            lat: Decimal,
            lon: Decimal,
        ) -> Result<Decimal, Box<dyn std::error::Error>>;
        async fn current_temp(
            &self,
            locations: &[LocationSpec],
        ) -> Result<Vec<Decimal>, Box<dyn std::error::Error>>;
        async fn forecast(
            &self,
            locations: &[LocationSpec],
            date: NaiveDate,
        ) -> Result<Vec<Forecast>, Box<dyn std::error::Error>>;
    }
}

type MockHandler = WeatherHandler<MockWeatherClient>;

#[fixture]
fn mock_client() -> MockWeatherClient {
    let client = MockWeatherClient::new();
    client
}

#[fixture]
fn mock_handler(mock_client: MockWeatherClient) -> MockHandler {
    WeatherHandler::new_with_client(mock_client)
}

fn app_client(client: MockWeatherClient) -> TestServer {
    app(WeatherHandler::new_with_client(client))
}

#[fixture]
fn app(mock_handler: MockHandler) -> TestServer {
    TestServer::new(crate::routes_v2(mock_handler).split_for_parts().0)
}

mod current_weather_tests {
    use super::*;

    #[rstest]
    #[case(52.52, 13.405, "23.5")] // Berlin coordinates
    #[case(40.7128, -74.006, "18.2")] // New York coordinates
    #[case(-33.8688, 151.2093, "28.7")] // Sydney coordinates
    #[tokio::test]
    async fn test_get_current_weather_success(
        #[case] lat: f64,
        #[case] lon: f64,
        #[case] expected_temp: &str,
        app: TestServer,
        mut mock_client: MockWeatherClient,
    ) {
        // Arrange
        let expected_decimal = Decimal::from_str_exact(expected_temp).unwrap();

        mock_client
            .expect_location_current_temperature()
            .with(
                eq(Decimal::from_f64_retain(lat).unwrap()),
                eq(Decimal::from_f64_retain(lon).unwrap()),
            )
            .times(1)
            .returning(move |_, _| Ok(expected_decimal));

        // Act
        let response = app
            .get("/weather")
            .add_query_param("lat", &lat.to_string())
            .add_query_param("lon", &lon.to_string())
            .await;

        // Assert
        response.assert_status(StatusCode::OK);

        let body: SuccessResponse<CurrentWeather> = response.json();
        assert_eq!(body.code, StatusCode::OK.as_u16());
        assert_eq!(
            body.data.temperature,
            Decimal::from_str_exact(expected_temp).unwrap()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_current_weather_missing_params(app: TestServer) {
        let response = app.get("/weather").add_query_param("lon", "13.405").await;

        response.assert_status(StatusCode::BAD_REQUEST);
        let body: StatusResponse = response.json();
        assert!(body.message.contains("invalid coordinates"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_current_weather_invalid_params(app: TestServer) {
        let response = app
            .get("/weather")
            .add_query_param("lat", "invalid")
            .add_query_param("lon", "13.405")
            .await;

        response.assert_status(StatusCode::BAD_REQUEST);
        let body: StatusResponse = response.json();
        assert!(body.message.contains("invalid coordinates"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_current_weather_service_error(
        app: TestServer,
        mut mock_client: MockWeatherClient,
    ) {
        mock_client
            .expect_location_current_temperature()
            .with(
                eq(Decimal::from_f64_retain(52.52).unwrap()),
                eq(Decimal::from_f64_retain(13.405).unwrap()),
            )
            .times(1)
            .returning(|_, _| {
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Weather service unavailable",
                )))
            });

        // Act
        let response = app
            .get("/weather")
            .add_query_param("lat", "52.52")
            .add_query_param("lon", "13.405")
            .await;

        // Assert
        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
        let body: StatusResponse = response.json();
        assert!(body.message.contains("Weather service unavailable"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_current_weather_extreme_coordinates(
        app: TestServer,
        mut mock_client: MockWeatherClient,
    ) {
        let extreme_temp = Decimal::from_str_exact("-89.2").unwrap();

        mock_client
            .expect_location_current_temperature()
            .with(
                eq(Decimal::from_i128_with_scale(-90, 0)),
                eq(Decimal::from_i128_with_scale(180, 0)),
            )
            .times(1)
            .returning(move |_, _| Ok(extreme_temp));

        let response = app
            .get("/weather")
            .add_query_param("lat", "-90")
            .add_query_param("lon", "180")
            .await;

        // Assert
        response.assert_status(StatusCode::OK);
        let body: SuccessResponse<CurrentWeather> = response.json();
        assert_eq!(
            body.data.temperature,
            Decimal::from_str_exact("-89.2").unwrap()
        );
    }
}

mod forecast_tests {
    use super::*;
    use chrono::NaiveDate;

    #[rstest]
    #[tokio::test]
    async fn test_get_forecast_success(mut mock_client: MockWeatherClient) {
        let forecast_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let expected_forecast = vec![
            Forecast {
                temperature: Decimal::from_str_exact("22.5").unwrap(),
                wind_direction: "NW".to_string(),
                humidity: 65,
                precipitation: Decimal::from_str_exact("0.0").unwrap(),
            },
            Forecast {
                temperature: Decimal::from_str_exact("18.3").unwrap(),
                wind_direction: "SE".to_string(),
                humidity: 78,
                precipitation: Decimal::from_str_exact("2.5").unwrap(),
            },
        ];

        mock_client
            .expect_forecast()
            .with(
                eq(vec![LocationSpec::City("Berlin".to_string())]),
                eq(forecast_date),
            )
            .times(1)
            .returning(move |_, _| Ok(expected_forecast.clone()));

        let app = app_client(mock_client);

        // Act
        let response = app
            .get("/forecast")
            .add_query_param("city", "Berlin")
            .add_query_param("date", "2024-01-15")
            .await;

        // Assert
        response.assert_status(StatusCode::OK);
        let body: SuccessResponse<Vec<Forecast>> = response.json();
        assert_eq!(body.data.len(), 2);
        assert_eq!(
            body.data[0].temperature,
            Decimal::from_str_exact("22.5").unwrap()
        );
        assert_eq!(body.data[0].wind_direction, "NW");
        assert_eq!(
            body.data[1].temperature,
            Decimal::from_str_exact("18.3").unwrap()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_forecast_coordinates_success(
        app: TestServer,
        mut mock_client: MockWeatherClient,
    ) {
        let forecast_date = NaiveDate::from_ymd_opt(2024, 6, 21).unwrap();

        let expected_forecast = vec![Forecast {
            temperature: Decimal::from_str_exact("30.1").unwrap(),
            wind_direction: "S".to_string(),
            humidity: 45,
            precipitation: Decimal::from_str_exact("0.0").unwrap(),
        }];

        mock_client
            .expect_forecast()
            .with(
                eq(vec![LocationSpec::Coordinates {
                    lat: Decimal::from_str_exact("35.6762").unwrap(),
                    lon: Decimal::from_str_exact("139.6503").unwrap(),
                }]),
                eq(forecast_date),
            )
            .times(1)
            .returning(move |_, _| Ok(expected_forecast.clone()));

        // Act - Tokyo coordinates
        let response = app
            .get("/forecast")
            .add_query_param("lat", "35.6762")
            .add_query_param("lon", "139.6503")
            .add_query_param("date", "2024-06-21")
            .await;

        // Assert
        response.assert_status(StatusCode::OK);
        let body: SuccessResponse<Vec<Forecast>> = response.json();
        assert_eq!(body.data.len(), 1);
        assert_eq!(
            body.data[0].temperature,
            Decimal::from_str_exact("30.1").unwrap()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_multi_forecast_success(mut mock_client: MockWeatherClient) {
        let expected_forecasts = vec![
            Forecast {
                temperature: Decimal::from_str_exact("25.0").unwrap(),
                wind_direction: "NE".to_string(),
                humidity: 55,
                precipitation: Decimal::from_str_exact("0.0").unwrap(),
            },
            Forecast {
                temperature: Decimal::from_str_exact("15.5").unwrap(),
                wind_direction: "SW".to_string(),
                humidity: 82,
                precipitation: Decimal::from_str_exact("5.2").unwrap(),
            },
        ];

        let request_body = json!({
            "locations": [
                {"city": "London"},
                {"lat": "48.8566", "lon": "2.3522"}  // Paris
            ],
            "date": "2024-03-20"
        });

        mock_client
            .expect_forecast()
            .with(
                eq(vec![
                    LocationSpec::City("London".to_string()),
                    LocationSpec::Coordinates {
                        lat: Decimal::from_str_exact("48.8566").unwrap(),
                        lon: Decimal::from_str_exact("2.3522").unwrap(),
                    },
                ]),
                eq(NaiveDate::from_ymd_opt(2024, 3, 20).unwrap()),
            )
            .times(1)
            .returning(move |_, _| Ok(expected_forecasts.clone()));

        let app = app_client(mock_client);

        // Act
        let response = app.post("/forecast/multi").json(&request_body).await;

        // Assert
        response.assert_status(StatusCode::OK);
        let body: SuccessResponse<Vec<Forecast>> = response.json();
        assert_eq!(body.data.len(), 2);
        assert_eq!(
            body.data[0].temperature,
            Decimal::from_str_exact("25.0").unwrap()
        );
        assert_eq!(body.data[1].wind_direction, "SW");
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_multi_forecast_empty_locations(
        app: TestServer,
        mut mock_client: MockWeatherClient,
    ) {
        mock_client
            .expect_forecast()
            .with(eq(vec![]), eq(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()))
            .times(1)
            .returning(|_, _| Ok(vec![]));

        let request_body = json!({
            "locations": [],
            "date": "2024-01-01"
        });

        // Act
        let response = app.post("/forecast/multi").json(&request_body).await;

        // Assert
        response.assert_status(StatusCode::OK);
        let body: SuccessResponse<Vec<Forecast>> = response.json();
        assert!(body.data.is_empty());
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_multi_forecast_invalid_request(app: TestServer) {
        let invalid_request = json!({
            "wrong_field": "value"
        });

        let response = app.post("/forecast/multi").json(&invalid_request).await;

        // Assert - should get 422 Unprocessable Entity or 400 Bad Request
        assert!(response.status_code().is_client_error());
    }
}

mod edge_cases {
    use std::sync::Arc;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_decimal_precision_handling(app: TestServer) {
        // Arrange
        let mut mock_client = MockWeatherClient::new();
        let precise_temp = Decimal::from_str_exact("23.456").unwrap();

        mock_client
            .expect_location_current_temperature()
            .with(
                eq(Decimal::from_str_exact("52.520008").unwrap()),
                eq(Decimal::from_str_exact("13.404954").unwrap()),
            )
            .times(1)
            .returning(move |_, _| Ok(precise_temp));

        // Act - high precision coordinates
        let response = app
            .get("/weather")
            .add_query_param("lat", "52.520008")
            .add_query_param("lon", "13.404954")
            .await;

        // Assert
        response.assert_status(StatusCode::OK);
        let body: SuccessResponse<CurrentWeather> = response.json();
        assert_eq!(
            body.data.temperature,
            Decimal::from_str_exact("23.456").unwrap()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_zero_temperature_handling(app: TestServer) {
        // Arrange
        let mut mock_client = MockWeatherClient::new();

        mock_client
            .expect_location_current_temperature()
            .with(
                eq(Decimal::from_str_exact("0").unwrap()),
                eq(Decimal::from_str_exact("0").unwrap()),
            )
            .times(1)
            .returning(|_, _| Ok(Decimal::ZERO));

        // Act - zero coordinates (Null Island)
        let response = app
            .get("/weather")
            .add_query_param("lat", "0")
            .add_query_param("lon", "0")
            .await;

        // Assert
        response.assert_status(StatusCode::OK);
        let body: SuccessResponse<CurrentWeather> = response.json();
        assert_eq!(body.data.temperature, Decimal::ZERO);
    }

    #[rstest]
    #[tokio::test]
    async fn test_concurrent_requests() {
        // Arrange
        let mut mock_client = MockWeatherClient::new();

        mock_client
            .expect_location_current_temperature()
            .times(3)
            .returning(|lat, lon| {
                let temp = if lat > Decimal::ZERO {
                    Decimal::from_str_exact("25.0").unwrap()
                } else {
                    Decimal::from_str_exact("10.0").unwrap()
                };
                Ok(temp)
            });

        let app = Arc::new(app_client(mock_client));

        // Act - simulate concurrent requests
        let mut set = tokio::task::JoinSet::new();
        for i in 0..3 {
            let app = app.clone();
            set.spawn(async move {
                app.get("/weather")
                    .add_query_param("lat", if i % 2 == 0 { "52.52" } else { "-33.86" })
                    .add_query_param("lon", "13.405")
                    .await
            });
        }

        for response in set.join_all().await {
            response.assert_status(StatusCode::OK);
        }
    }
}

#[cfg(false)]
mod integration_scenarios {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_weather_workflow_coordinates_to_forecast() {
        // Arrange
        let mut mock_client = MockWeatherClient::new();

        // Mock current weather
        mock_handler
            .expect_location_current_temperature()
            .with(
                eq(Decimal::from_str_exact("48.8566").unwrap()),
                eq(Decimal::from_str_exact("2.3522").unwrap()),
            )
            .times(1)
            .returning(|_, _| Ok(Decimal::from_str_exact("20.5").unwrap()));

        // Mock forecast
        mock_handler
            .expect_forecast()
            .with(
                eq(vec![LocationSpec::City("Paris".to_string())]),
                eq(NaiveDate::from_ymd_opt(2024, 7, 14).unwrap()),
            )
            .times(1)
            .returning(|_, _| {
                Ok(vec![Forecast {
                    temperature: Decimal::from_str_exact("28.0").unwrap(),
                    wind_direction: "W".to_string(),
                    humidity: 40,
                    precipitation: Decimal::from_str_exact("0.0").unwrap(),
                }])
            });

        let app = create_test_app(mock_handler);

        // Act - Get current weather
        let weather_response = app
            .get("/weather")
            .add_query_param("lat", "48.8566")
            .add_query_param("lon", "2.3522")
            .await;

        weather_response.assert_status(StatusCode::OK);

        // Act - Get forecast for Bastille Day
        let forecast_response = app
            .get("/forecast")
            .add_query_param("city", "Paris")
            .add_query_param("date", "2024-07-14")
            .await;

        // Assert
        forecast_response.assert_status(StatusCode::OK);
        let forecast_body: SuccessResponse<Vec<Forecast>> = forecast_response.json();
        assert!(!forecast_body.data.is_empty());
        assert_eq!(
            forecast_body.data[0].temperature,
            Decimal::from_str_exact("28.0").unwrap()
        );
    }
}
