use crate::server::app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn post_and_read(app: axum::Router, body: serde_json::Value) -> (StatusCode, String) {
    let res = app
        .oneshot(
            Request::post("/api/rewards-simulator")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes).to_string();
    (status, text)
}

#[tokio::test]
async fn api_happy_path() {
    let app = app();
    let scale = 1_000_000u64;
    let body = serde_json::json!({
      "cgp_leftovers": {},
      "solar_farms": [
        {
          "farm_id": "A",
          "asset_id": "usdg",
          "region_id": "utah",
          "weekly_carbon_credits": scale,
          "protocol_deposit_value": 10000u64 * scale,
          "assets_required": 10000u64 * scale,
          "rewards_address": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
          "first_week": 96,
          "weeks_alive": 2
        }
      ]
    });
    let (status, text) = post_and_read(app, body).await;
    if status != StatusCode::OK {
        panic!("expected 200 OK, got {status} with body: {text}");
    }
}

#[tokio::test]
async fn api_validation_error() {
    let app = app();
    let body = serde_json::json!({
      "cgp_leftovers": {},
      "solar_farms": []
    });
    let (status, text) = post_and_read(app, body).await;
    if status != StatusCode::BAD_REQUEST {
        panic!("expected 400 BAD_REQUEST, got {status} with body: {text}");
    }
}
