use rust_decimal::Decimal;
use serde_derive::{Deserialize, Serialize};
use utoipa::{IntoResponses, ToSchema};

#[derive(Serialize, Deserialize, ToSchema, IntoResponses)]
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

#[derive(Debug, Deserialize, ToSchema, Eq, PartialEq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case", untagged)]
pub enum LocationSpec {
    Coordinates { lat: Decimal, lon: Decimal },
    City(String),
}

impl LocationSpec {
    pub fn from_coord_str(lat: &str, lon: &str) -> Option<Self> {
        Decimal::from_str_exact(lat)
            .ok()
            .zip(Decimal::from_str_exact(lon).ok())
            .map(|(lat, lon)| Self::Coordinates { lat, lon })
    }
}
