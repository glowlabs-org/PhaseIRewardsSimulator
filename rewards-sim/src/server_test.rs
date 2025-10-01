use crate::server::app;
use crate::test_utils::post_and_read;
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::FromPrimitive;

#[tokio::test]
async fn api_happy_path() {
    let app = app();
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let ia = scale.to_string();
    let pd = (BigInt::from_u64(10_000).unwrap() * &scale).to_string();
    let ar = (BigInt::from_u64(10_000).unwrap() * &scale).to_string();

    let body = serde_json::json!({
      "cgpLeftovers": {},
      "solarFarms": [
        {
          "farmId": "A",
          "assetId": "usdg",
          "regionId": "utah",
          "netWeeklyImpactAssets": ia,
          "protocolDepositValue": pd,
          "assetsRequired": ar,
          "rewardsAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
          "firstWeek": 96,
          "weeksAlive": 2
        }
      ]
    });
    let (status_basic, text_basic) =
        post_and_read(app.clone(), "/api/rewards-simulator", body.clone()).await;
    if status_basic != StatusCode::OK {
        panic!("expected 200 OK (basic), got {status_basic} with body: {text_basic}");
    }
    let (status_det, text_det) = post_and_read(app, "/api/rewards-simulator-detailed", body).await;
    if status_det != StatusCode::OK {
        panic!("expected 200 OK (detailed), got {status_det} with body: {text_det}");
    }
}

#[tokio::test]
async fn api_validation_error() {
    let app = app();
    let body = serde_json::json!({
      "cgpLeftovers": {},
      "solarFarms": []
    });
    let (status_basic, text_basic) =
        post_and_read(app.clone(), "/api/rewards-simulator", body.clone()).await;
    if status_basic != StatusCode::BAD_REQUEST {
        panic!("expected 400 BAD_REQUEST (basic), got {status_basic} with body: {text_basic}");
    }
    let (status_det, text_det) = post_and_read(app, "/api/rewards-simulator-detailed", body).await;
    if status_det != StatusCode::BAD_REQUEST {
        panic!("expected 400 BAD_REQUEST (detailed), got {status_det} with body: {text_det}");
    }
}
