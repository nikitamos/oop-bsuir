mod google;
mod open_weather;
mod weather_data_client;

use models::weather::{Coordinates, LocationSpec};
use reqwest::StatusCode;
use std::error::Error;

pub use google::*;
pub use open_weather::*;
pub use weather_data_client::*;

#[derive(Debug)]
pub struct BadStatusError {
    code: StatusCode,
    message: String,
}

impl BadStatusError {
    pub fn new(service: &str, code: StatusCode) -> Self {
        Self {
            code,
            message: format!("{service} returned bad status: {code}"),
        }
    }
    pub fn code(&self) -> StatusCode {
        self.code
    }
}

impl std::fmt::Display for BadStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for BadStatusError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Default)]
pub(crate) struct LocationNormalizer;

impl LocationNormalizer {
    pub(crate) fn normalize_locations(
        &self,
        locations: &[LocationSpec],
    ) -> impl Iterator<Item = Option<Coordinates>> {
        locations.iter().map(|l| match l {
            LocationSpec::Coordinates { lat, lon } => Some(Coordinates {
                lat: *lat,
                lon: *lon,
            }),
            LocationSpec::City { city } => self.map_city_to_coords(city),
        })
    }
    pub(crate) fn map_city_to_coords(&self, city: &str) -> Option<Coordinates> {
        match city.to_lowercase().as_str() {
            "minsk" => Coordinates::from_str("53.9006", "27.5590"),
            "london" => Coordinates::from_str("51.5072", "-0.1276"),
            "tokyo" => Coordinates::from_str("35.6764", "139.6500"),
            "shanghai" => Coordinates::from_str("31.2304", "121.4737"),
            "warsaw" => Coordinates::from_str("52.2297", "21.0122"),
            _ => {
                log::error!("Unknown city: {city}");
                None
            }
        }
    }
}

fn deg_to_cardinal(deg: f64) -> String {
    const DIRS: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    let idx = ((deg + 22.5) % 360.0) as usize / 45;
    DIRS[idx].to_string()
}
