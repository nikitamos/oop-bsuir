use clients::WeatherDataClient;
use models::weather::CurrentWeather;
use rust_decimal::Decimal;

#[derive(Clone)]
pub struct CurrentWeatherController<T: WeatherDataClient> {
    pub client: T,
}

impl<T: WeatherDataClient> CurrentWeatherController<T> {
    pub fn new(client: T) -> Self {
        CurrentWeatherController { client }
    }
    pub async fn get_current_weather(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> anyhow::Result<CurrentWeather> {
        Ok(CurrentWeather {
            temperature: self.client.location_current_temperature(lat, lon).await?,
        })
    }
}
