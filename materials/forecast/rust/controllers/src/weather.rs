use clients::WeatherDataClient;
use models::weather::{CurrentWeather, Forecast, LocationSpec};
use rust_decimal::Decimal;

#[derive(Clone)]
pub struct CurrentWeatherController<T: WeatherDataClient> {
    pub client: T,
}

impl<T: WeatherDataClient> CurrentWeatherController<T> {
    pub fn new(client: T) -> Self {
        CurrentWeatherController { client }
    }
    #[deprecated]
    pub async fn get_current_weather(
        &self,
        lat: Decimal,
        lon: Decimal,
    ) -> anyhow::Result<CurrentWeather> {
        self.get_current_weather_at(&[LocationSpec::Coordinates { lat, lon }])
            .await
            .map(|mut v| v.remove(0))
    }
    pub async fn get_current_weather_at(
        &self,
        locations: &[LocationSpec],
    ) -> anyhow::Result<Vec<CurrentWeather>> {
        println!("calling current_temp with {locations:?}");
        self.client.current_temp(locations).await.map(|v| {
            v.into_iter()
                .map(|temperature| CurrentWeather { temperature })
                .collect()
        })
    }
    pub async fn get_forecast_at(
        &self,
        locations: &[LocationSpec],
        date: chrono::NaiveDate,
    ) -> anyhow::Result<Vec<Forecast>> {
        self.client.forecast(locations, date).await
    }
}
