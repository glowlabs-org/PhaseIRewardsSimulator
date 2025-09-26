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

#[cfg(test)]
mod http_tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn api_happy_path() {
        let app = crate::server::app();
        let body = serde_json::json!({
          "cgp_leftovers": {},
          "solar_farms": [
            {
              "farm_id": "A",
              "asset_id": "usdg",
              "region_id": "utah",
              "weekly_carbon_credits": "1",
              "protocol_deposit_value": "10000",
              "assets_required": "10000",
              "rewards_address": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "first_week": 96,
              "weeks_alive": 2
            }
          ]
        });
        let res = app
            .oneshot(
                Request::post("/api/rewards-simulator")
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_validation_error() {
        let app = crate::server::app();
        let body = serde_json::json!({
          "cgp_leftovers": {},
          "solar_farms": []
        });
        let res = app
            .oneshot(
                Request::post("/api/rewards-simulator")
                    .header("content-type", "application/json")
                    .body(serde_json::to_vec(&body).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
