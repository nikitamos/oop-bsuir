#![feature(coverage_attribute)]
use std::sync::Arc;

use clients::{OpenWeatherClient, WeatherDataClient};
use controllers::weather::CurrentWeatherController;
use models::schemas::date_yyyy_mm_dd;
use models::weather::LocationSpec;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::utils;
use utoipa::{IntoParams, ToSchema, openapi::Schema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub mod forecast;
pub mod weather;

#[derive(Deserialize, Serialize, Default, ToSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WeatherProvider {
    #[default]
    OpenWeatherMap,
    Google,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in=Query)]
pub struct GetCurrentWeatherQuery {
    /// Latitude
    #[param(default = "18.300231990440125")]
    lat: Decimal,
    /// Longitude
    #[param(default = "-64.8251590359234")]
    lon: Decimal,
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in=Query)]
pub struct SingleCurrentWeatherQuery {
    #[serde(flatten)]
    location: LocationSpec,
    provider: WeatherProvider,
}

#[derive(Debug, Deserialize, Serialize, IntoParams, ToSchema)]
#[into_params(parameter_in=Query)]
pub struct SingleForecastQuery {
    #[serde(flatten)]
    pub location: LocationSpec,
    #[serde(default = "WeatherProvider::default")]
    pub provider: WeatherProvider,
    #[param(example = "2026-09-05")]
    pub date: chrono::NaiveDate,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MultiWeatherRequest {
    locations: Vec<LocationSpec>,
    #[serde(default = "WeatherProvider::default")]
    provider: WeatherProvider,
}

#[derive(Deserialize, ToSchema)]
pub struct MultiForecastRequest {
    locations: Vec<LocationSpec>,
    #[serde(default = "WeatherProvider::default")]
    provider: WeatherProvider,
    #[schema(example = "2026-09-05")]
    date: chrono::NaiveDate,
}

pub fn routes_v2<C>(handler: WeatherHandler<C>) -> OpenApiRouter
where
    C: WeatherDataClient + Send + Sync + 'static,
{
    OpenApiRouter::default()
        .routes(routes!(weather::handle_get_current_weather))
        .routes(routes!(weather::handle_get_current_weather2))
        .routes(routes!(weather::handle_get_multi_weather))
        .routes(routes!(forecast::handle_get_forecast))
        .routes(routes!(forecast::handle_get_multi_forecast))
        .with_state(handler)
}

pub struct WeatherHandler<C: WeatherDataClient> {
    pub controller: Arc<CurrentWeatherController<C>>,
}

impl<C: WeatherDataClient> Clone for WeatherHandler<C> {
    fn clone(&self) -> Self {
        Self {
            controller: Arc::clone(&self.controller),
        }
    }
}

impl<C: WeatherDataClient> WeatherHandler<C> {
    pub fn new_with_client(client: C) -> Self {
        Self {
            controller: Arc::new(CurrentWeatherController { client }),
        }
    }
    pub fn dispatch_provider(&self, provider: WeatherProvider) -> &CurrentWeatherController<C> {
        match provider {
            WeatherProvider::OpenWeatherMap => &self.controller,
            WeatherProvider::Google => todo!(),
        }
    }
}

impl WeatherHandler<OpenWeatherClient> {
    pub fn new() -> Self {
        Self::new_with_client(OpenWeatherClient::new(
            utils::getenv("OPENWEATHER_API_KEY", String::new()),
            utils::getenv("OPENWEATHER_BASE_URL", String::new()),
        ))
    }
}

#[macro_export]
macro_rules! accept {
    ($wrapper:ident from $var:expr) => {
        match $var {
            Ok($wrapper(val)) => val,
            Err(err) => return StatusResponse::new(err.status(), &err.to_string()).into_response(),
        }
    };
}

#[cfg(test)]
pub mod tests;
