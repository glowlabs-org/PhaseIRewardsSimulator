use crate::errors::SimError;
use crate::models::InputData;
use crate::simulator::simulate;
use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use serde_json::json;

pub fn app() -> Router {
    Router::new().route("/api/rewards-simulator", post(sim_handler))
}

async fn sim_handler(Json(input): Json<InputData>) -> Result<impl IntoResponse, AppError> {
    let out = simulate(input)?;
    Ok((StatusCode::OK, Json(out)))
}

#[derive(Debug)]
pub struct AppError(pub SimError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let msg = self.0.to_string();
        let status = match self.0 {
            SimError::Validation(_) => StatusCode::BAD_REQUEST,
            SimError::Algorithm(_) => StatusCode::UNPROCESSABLE_ENTITY,
            SimError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = Json(json!({ "error": msg }));
        (status, body).into_response()
    }
}

impl From<SimError> for AppError {
    fn from(value: SimError) -> Self {
        AppError(value)
    }
}
