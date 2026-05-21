mod open_weather;
mod weather_data_client;

use reqwest::StatusCode;
use std::error::Error;

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
