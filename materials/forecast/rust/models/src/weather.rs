use rust_decimal::Decimal;
use serde_derive::{Deserialize, Serialize};
use utoipa::{IntoParams, IntoResponses, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoResponses, PartialEq, Eq)]
#[response(status = 200)]
pub struct CurrentWeather {
    #[schema(examples("0.0"))]
    pub temperature: Decimal,
}

#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone)]
pub struct Forecast {
    pub temperature: Decimal,
    pub wind_direction: String,
    pub humidity: u32,
    pub precipitation: Decimal,
}

pub mod owm {
    #[derive(serde::Deserialize, Debug)]
    pub struct OpenWeatherForecastResponse {
        pub list: Vec<ForecastItem>,
    }

    #[derive(serde::Deserialize, Debug)]
    pub struct ForecastItem {
        pub dt: i64,
        pub main: MainData,
        pub wind: WindData,
        #[serde(default)]
        pub rain: Option<RainData>,
        #[serde(default)]
        pub pop: Option<f64>,
    }

    #[derive(serde::Deserialize, Debug)]
    pub struct MainData {
        pub temp: f64,
        pub humidity: u32,
    }

    #[derive(serde::Deserialize, Debug)]
    pub struct WindData {
        #[serde(default)]
        pub deg: Option<f64>,
    }

    #[derive(serde::Deserialize, Debug)]
    pub struct RainData {
        #[serde(rename = "3h", default)]
        pub hours_3: Option<f64>,
    }
}

pub mod google {
    use rust_decimal::Decimal;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct CurrentConditions {
        pub temperature: Temperature,
    }
    #[derive(Debug, Deserialize)]
    pub struct Temperature {
        pub degrees: Decimal,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GoogleForecastResponse {
        pub forecast_days: Vec<ForecastDay>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ForecastDay {
        pub daytime_forecast: DaytimeForecast,
        pub nighttime_forecast: NighttimeForecast,
        pub max_temperature: Temperature,
        pub min_temperature: Temperature,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct DaytimeForecast {
        pub relative_humidity: u32,
        pub precipitation: Precipitation,
        pub wind: Wind,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct NighttimeForecast {
        pub relative_humidity: u32,
        pub precipitation: Precipitation,
        pub wind: Wind,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Precipitation {
        #[serde(default)]
        pub qpf: Option<QuantitativePrecipitationForecast>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct QuantitativePrecipitationForecast {
        pub quantity: Decimal,
        #[serde(default)]
        pub unit: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Wind {
        pub direction: WindDirection,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct WindDirection {
        pub degrees: Option<f64>,
        pub cardinal: Option<String>,
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct Coordinates {
    pub lat: Decimal,
    pub lon: Decimal,
}

impl Coordinates {
    pub fn from_str(lat: &str, lon: &str) -> Option<Self> {
        Decimal::from_str_exact(lat)
            .ok()
            .zip(Decimal::from_str_exact(lon).ok())
            .map(|(lat, lon)| Self { lat, lon })
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Eq, PartialEq, PartialOrd, Ord, Clone)]
#[serde(rename_all = "snake_case", untagged)]
pub enum LocationSpec {
    Coordinates { lat: Decimal, lon: Decimal },
    City { city: String },
}

impl LocationSpec {
    pub fn from_coord_str(lat: &str, lon: &str) -> Option<Self> {
        Decimal::from_str_exact(lat)
            .ok()
            .zip(Decimal::from_str_exact(lon).ok())
            .map(|(lat, lon)| Self::Coordinates { lat, lon })
    }
    pub fn from_city(city: &str) -> Self {
        Self::City {
            city: city.to_string(),
        }
    }
}

pub fn segregate_locations(locations: &[LocationSpec]) -> (Vec<String>, Vec<(Decimal, Decimal)>) {
    let mut cities = vec![];
    let mut coords = vec![];

    for l in locations {
        match l {
            LocationSpec::Coordinates { lat, lon } => coords.push((*lat, *lon)),
            LocationSpec::City { city } => cities.push(city.clone()),
        }
    }

    (cities, coords)
}
