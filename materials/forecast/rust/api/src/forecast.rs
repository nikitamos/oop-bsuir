use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
};
use clients::WeatherDataClient;
use models::weather::{CurrentWeather, Forecast};
use shared::responses::{StatusResponse, SuccessResponse};

use crate::{MultiForecastRequest, SingleForecastQuery, WeatherHandler};

#[utoipa::path(get,
    path = "/forecast",
    params(SingleForecastQuery),
    responses(
        (status=200, body=SuccessResponse<CurrentWeather>),
        (status=400, body=StatusResponse),
        (status=500, body=StatusResponse)
    )
)]
pub async fn handle_get_forecast<C: WeatherDataClient>(
    State(h): State<WeatherHandler<C>>,
    query: Result<Query<SingleForecastQuery>, QueryRejection>,
) -> axum::response::Response {
    todo!()
}

#[utoipa::path(post,
    path = "/forecast/multi",
    request_body = MultiForecastRequest,
    responses(
        (status=200, body=SuccessResponse<Forecast>),
        (status=400, body=StatusResponse),
        (status=500, body=StatusResponse)
    )
)]
// #[axum::debug_handler]
pub async fn handle_get_multi_forecast<C: WeatherDataClient>(
    State(h): State<WeatherHandler<C>>,
    Json(x): Json<MultiForecastRequest>,
) -> axum::response::Response {
    todo!()
}
