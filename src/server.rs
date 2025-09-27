use crate::errors::SimError;
use crate::models::InputData;
use crate::simulator::{simulate_with_diagnostics, SimulationDiagnostics};
use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use serde_json::json;

pub fn app() -> Router {
    Router::new()
        .route("/api/rewards-simulator", post(sim_handler))
        .route(
            "/api/rewards-simulator-detailed",
            post(sim_detailed_handler),
        )
}

async fn sim_handler(Json(input): Json<InputData>) -> Result<Response, AppError> {
    match simulate_with_diagnostics(input) {
        Ok(SimulationDiagnostics { output, errors, .. }) => {
            if errors.is_empty() {
                Ok((StatusCode::OK, Json(output)).into_response())
            } else {
                let body = json!({ "errors": errors, "output": output });
                Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response())
            }
        }
        Err(e) => Err(AppError(e)),
    }
}

async fn sim_detailed_handler(Json(input): Json<InputData>) -> Result<Response, AppError> {
    match simulate_with_diagnostics(input) {
        Ok(diag) => {
            if diag.errors.is_empty() {
                Ok((StatusCode::OK, Json(diag)).into_response())
            } else {
                Ok((StatusCode::UNPROCESSABLE_ENTITY, Json(diag)).into_response())
            }
        }
        Err(e) => Err(AppError(e)),
    }
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
        let body = axum::Json(serde_json::json!({ "error": msg }));
        (status, body).into_response()
    }
}

impl From<SimError> for AppError {
    fn from(value: SimError) -> Self {
        AppError(value)
    }
}
