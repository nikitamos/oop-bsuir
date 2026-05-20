use super::SingleCurrentWeatherQuery;
use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
    http::StatusCode,
    response::IntoResponse,
};
use clients::WeatherDataClient;
use models::weather::CurrentWeather;
use shared::responses::{self, StatusResponse, SuccessResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    GetCurrentWeatherQuery, MultiForecastRequest, MultiWeatherRequest, SingleForecastQuery,
    WeatherHandler,
};

pub fn routers() -> OpenApiRouter {
    OpenApiRouter::default()
        .routes(routes!(handle_get_current_weather))
        .with_state(WeatherHandler::new())
}

#[utoipa::path(get,
    path = "/weather",
    params(GetCurrentWeatherQuery),
    summary = "Get Current Weather",
    description = "Returns current weather for given coordinates",
    tags = ["weather"],
    responses(
         (status=200, body=SuccessResponse<CurrentWeather>),
         (status=400, body=StatusResponse),
         (status=500, body=StatusResponse)
    ),
)]
pub async fn handle_get_current_weather<C: WeatherDataClient>(
    State(h): State<WeatherHandler<C>>,
    query: Result<Query<GetCurrentWeatherQuery>, QueryRejection>,
) -> axum::response::Response {
    if query.is_err() {
        return responses::StatusResponse {
            code: StatusCode::BAD_REQUEST.as_u16(),
            message: "invalid coordinates".to_string(),
        }
        .into_response();
    }
    let Query(GetCurrentWeatherQuery { lat, lon }) = query.unwrap();
    match h.controller.get_current_weather(lat, lon).await {
        Ok(data) => responses::SuccessResponse::new(StatusCode::OK, data).into_response(),
        Err(err) => responses::StatusResponse {
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: err.to_string(),
        }
        .into_response(),
    }
}

#[utoipa::path(get,
    path = "/weather2",
    params(SingleCurrentWeatherQuery),
    responses(
        (status=200, body=SuccessResponse<CurrentWeather>),
        (status=400, body=StatusResponse),
        (status=500, body=StatusResponse)
    )
)]
pub async fn handle_get_current_weather2<C: WeatherDataClient>(
    State(h): State<WeatherHandler<C>>,
    query: Result<Query<SingleCurrentWeatherQuery>, QueryRejection>,
) -> axum::response::Response {
    let params = match query {
        Ok(Query(q)) => q,
        Err(err) => {
            return StatusResponse::new(err.status(), "invalid locations").into_response();
        }
    };
    h.dispatch_provider(params.provider)
        .get_current_weather_at(&[params.location])
        .await
        .map(|mut v| SuccessResponse::new(StatusCode::OK, v.remove(0)))
        .map_err(|e| StatusResponse::new_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .into_response()
}

#[utoipa::path(post,
    path = "/weather2/multi",
    request_body = MultiWeatherRequest,
    responses(
        (status=200, body=SuccessResponse<CurrentWeather>),
        (status=400, body=StatusResponse),
        (status=500, body=StatusResponse)
    )
)]
pub async fn handle_get_multi_weather<C: WeatherDataClient>(
    State(h): State<WeatherHandler<C>>,
    Json(x): Json<MultiWeatherRequest>,
) -> axum::response::Response {
    todo!()
}
