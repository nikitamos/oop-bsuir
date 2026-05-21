use axum::{
    Json,
    extract::{
        Query, State,
        rejection::{JsonRejection, QueryRejection},
    },
    http::StatusCode,
    response::IntoResponse,
};
use clients::WeatherDataClient;
use models::weather::{CurrentWeather, Forecast};
use shared::responses::{StatusResponse, SuccessResponse};

use crate::{MultiForecastRequest, SingleForecastQuery, WeatherHandler, accept};

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
    let query = accept!(Query from query);
    h.dispatch_provider(query.provider)
        .get_forecast_at(&[query.location], query.date)
        .await
        .map(SuccessResponse::ok)
        .map_err(|e| StatusResponse::new_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .into_response()
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
    x: Result<Json<MultiForecastRequest>, JsonRejection>,
) -> axum::response::Response {
    let request = accept!(Json from x);
    h.dispatch_provider(request.provider)
        .get_forecast_at(&request.locations, request.date)
        .await
        .map(SuccessResponse::ok)
        .map_err(|e| StatusResponse::new_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .into_response()
}
