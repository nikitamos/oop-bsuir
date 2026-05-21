#![feature(coverage_attribute)]
#![feature(impl_trait_in_bindings)]
use std::sync::Arc;

use clients::{GoogleWeatherClient, OpenWeatherClient, WeatherDataClient};
use controllers::weather::CurrentWeatherController;
use models::weather::LocationSpec;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::utils;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

pub mod forecast;
pub mod weather;

#[derive(Deserialize, Clone, Serialize, Default, ToSchema, Debug)]
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

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
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

pub fn routes_v2(handler: WeatherHandler) -> OpenApiRouter {
    OpenApiRouter::default()
        .routes(routes!(weather::handle_get_current_weather))
        .routes(routes!(weather::handle_get_current_weather2))
        .routes(routes!(weather::handle_get_multi_weather))
        .routes(routes!(forecast::handle_get_forecast))
        .routes(routes!(forecast::handle_get_multi_forecast))
        .with_state(handler)
}

pub struct WeatherHandler {
    pub controller: Arc<CurrentWeatherController<dyn WeatherDataClient>>,
    pub google: Arc<CurrentWeatherController<dyn WeatherDataClient>>,
}

impl WeatherHandler {
    pub fn dispatch_provider(
        &self,
        provider: WeatherProvider,
    ) -> Arc<CurrentWeatherController<dyn WeatherDataClient>> {
        match provider {
            WeatherProvider::OpenWeatherMap => Arc::clone(&self.controller),
            WeatherProvider::Google => Arc::clone(&self.google),
        }
    }
}

impl Clone for WeatherHandler {
    fn clone(&self) -> Self {
        Self {
            controller: Arc::clone(&self.controller),
            google: Arc::clone(&self.google),
        }
    }
}

impl WeatherHandler {
    pub fn new_with_client<C: WeatherDataClient + 'static>(client: C) -> Self {
        let ctrl: Arc<CurrentWeatherController<dyn WeatherDataClient>> =
            Arc::new(CurrentWeatherController { client });
        Self {
            controller: Arc::clone(&ctrl),
            google: ctrl,
        }
    }
    pub fn new_with_clients<C: WeatherDataClient + 'static, G: WeatherDataClient + 'static>(
        owm: C,
        google: G,
    ) -> Self {
        Self {
            controller: Arc::new(CurrentWeatherController::new(owm)),
            google: Arc::new(CurrentWeatherController::new(google)),
        }
    }
}

impl WeatherHandler {
    pub fn new() -> Self {
        Self::new_with_clients(
            OpenWeatherClient::from_env(),
            GoogleWeatherClient::from_env(),
        )
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
