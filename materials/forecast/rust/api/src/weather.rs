use super::SingleCurrentWeatherQuery;
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
use models::weather::CurrentWeather;
use shared::responses::{self, StatusResponse, SuccessResponse};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{GetCurrentWeatherQuery, MultiWeatherRequest, WeatherHandler, accept};

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
    let GetCurrentWeatherQuery { lat, lon } = accept!(Query from query);
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
    let params = accept!(Query from query);
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
        (status=422, body=StatusResponse),
        (status=500, body=StatusResponse)
    )
)]
pub async fn handle_get_multi_weather<C: WeatherDataClient>(
    State(h): State<WeatherHandler<C>>,
    request: Result<Json<MultiWeatherRequest>, JsonRejection>,
) -> axum::response::Response {
    let request = accept!(Json from request);
    h.dispatch_provider(request.provider)
        .get_current_weather_at(&request.locations)
        .await
        .map(SuccessResponse::ok)
        .map_err(|e| StatusResponse::new_owned(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .into_response()
}
