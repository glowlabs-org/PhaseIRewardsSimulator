use axum::http::{Request, StatusCode};
use rewards_simulator::server::app;
use tower::ServiceExt;

#[tokio::test]
async fn api_happy_path() {
    let app = app();
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
    let app = app();
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
