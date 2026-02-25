use crate::server::app;
use crate::test_utils::post_and_read;
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde_json::Value;

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
          "regionId": 2,
          "netWeeklyImpactAssets": ia,
          "protocolDepositValue": pd,
          "assetsRequired": ar,
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
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

#[tokio::test]
async fn api_week_query_returns_single_object_and_404_when_absent() {
    let app = app();
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let ia = scale.to_string();
    let pd = (BigInt::from_u64(10_000).unwrap() * &scale).to_string();
    let ar = (BigInt::from_u64(10_000).unwrap() * &scale).to_string();

    // Farm active in weeks 96 and 97
    let body = serde_json::json!({
      "cgpLeftovers": {},
      "solarFarms": [
        {
          "farmId": "A",
          "assetId": "usdg",
          "regionId": 2,
          "netWeeklyImpactAssets": ia,
          "protocolDepositValue": pd,
          "assetsRequired": ar,
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 96,
          "weeksAlive": 2
        }
      ]
    });

    // Request a single week (96) and ensure the structure is the per-week object
    let (status_wk, text_wk) =
        post_and_read(app.clone(), "/api/rewards-simulator?week=96", body.clone()).await;
    assert_eq!(
        status_wk,
        StatusCode::OK,
        "expected 200 for single week request, got body: {text_wk}"
    );
    let v: Value = serde_json::from_str(&text_wk).expect("json");
    assert!(
        v.get("walletDistributions").is_some()
            && v.get("farmRewards").is_some()
            && v.get("regionData").is_some(),
        "expected single-week object with required fields, got: {v:?}"
    );

    // Request a week that doesn't exist => 404 Not Found
    let (status_missing, _text_missing) =
        post_and_read(app, "/api/rewards-simulator?week=123456", body).await;
    assert_eq!(
        status_missing,
        StatusCode::NOT_FOUND,
        "expected 404 for missing week"
    );
}

#[tokio::test]
async fn api_multi_asset_happy_path() {
    let app = app();
    let scale_18 = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let scale_6 = BigInt::from_u64(1_000_000).unwrap();

    let body = serde_json::json!({
      "cgpLeftovers": {},
      "solarFarms": [
        {
          "farmId": "MA1",
          "regionId": 3,
          "netWeeklyImpactAssets": (BigInt::from_u64(100).unwrap() * &scale_18).to_string(),
          "totalProtocolDepositValue": (BigInt::from_u64(100).unwrap() * &scale_6).to_string(),
          "assets": [{
              "assetId": "GLW",
              "assetsRequired": (BigInt::from_u64(100).unwrap() * &scale_18).to_string(),
              "assetsRequiredUSDC": (BigInt::from_u64(100).unwrap() * &scale_6).to_string(),
              "quotedByGvePricePerAsset": scale_6.to_string()
          }],
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 98,
          "weeksAlive": 5
        }
      ]
    });

    let (status, text) = post_and_read(app, "/api/rewards-simulator-multi-asset?week=98", body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "expected 200 OK (multi-asset), got body: {text}"
    );
    let v: Value = serde_json::from_str(&text).expect("json");
    assert!(
        v.get("farmRewards").is_some()
            && v.get("walletDistributions").is_some()
            && v.get("regionData").is_some(),
        "expected single-week multi-asset shape, got: {v:?}"
    );
}

#[tokio::test]
async fn api_multi_asset_rejects_noncanonical_asset_id_case() {
    let app = app();
    let scale_18 = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let scale_6 = BigInt::from_u64(1_000_000).unwrap();

    let body = serde_json::json!({
      "cgpLeftovers": {},
      "solarFarms": [
        {
          "farmId": "BADCASE",
          "regionId": 3,
          "netWeeklyImpactAssets": scale_18.to_string(),
          "totalProtocolDepositValue": (BigInt::from_u64(10).unwrap() * &scale_6).to_string(),
          "assets": [{
              "assetId": "glw",
              "assetsRequired": scale_18.to_string(),
              "assetsRequiredUSDC": (BigInt::from_u64(10).unwrap() * &scale_6).to_string(),
              "quotedByGvePricePerAsset": scale_6.to_string()
          }],
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 98,
          "weeksAlive": 5
        }
      ]
    });

    let (status, text) = post_and_read(app, "/api/rewards-simulator-multi-asset", body).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "expected 400 BAD_REQUEST (multi-asset invalid assetId case), got body: {text}"
    );
    assert!(
        text.contains("exact uppercase"),
        "expected canonical-case validation message, got: {text}"
    );
}

#[tokio::test]
async fn api_multi_asset_output_farms_week_returns_reduced_shape() {
    let app = app();
    let scale_18 = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let scale_6 = BigInt::from_u64(1_000_000).unwrap();

    let body = serde_json::json!({
      "cgpLeftovers": {},
      "outputFarms": ["F1"],
      "solarFarms": [
        {
          "farmId": "F1",
          "regionId": 1,
          "netWeeklyImpactAssets": scale_18.to_string(),
          "totalProtocolDepositValue": (BigInt::from_u64(10).unwrap() * &scale_6).to_string(),
          "assets": [{
              "assetId": "USDG",
              "assetsRequired": (BigInt::from_u64(10).unwrap() * &scale_6).to_string(),
              "assetsRequiredUSDC": (BigInt::from_u64(10).unwrap() * &scale_6).to_string(),
              "quotedByGvePricePerAsset": scale_6.to_string()
          }],
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 10,
          "weeksAlive": 5
        },
        {
          "farmId": "F2",
          "regionId": 1,
          "netWeeklyImpactAssets": scale_18.to_string(),
          "totalProtocolDepositValue": (BigInt::from_u64(10).unwrap() * &scale_6).to_string(),
          "assets": [{
              "assetId": "USDG",
              "assetsRequired": (BigInt::from_u64(10).unwrap() * &scale_6).to_string(),
              "assetsRequiredUSDC": (BigInt::from_u64(10).unwrap() * &scale_6).to_string(),
              "quotedByGvePricePerAsset": scale_6.to_string()
          }],
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 10,
          "weeksAlive": 5
        }
      ]
    });

    let (status, text) =
        post_and_read(app, "/api/rewards-simulator-multi-asset?week=10", body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "expected 200 OK (multi-asset filtered week), got body: {text}"
    );
    let v: Value = serde_json::from_str(&text).expect("json");
    assert!(v.get("farmRewards").is_some(), "expected farmRewards in filtered response");
    assert!(v.get("warnings").is_some(), "expected warnings in filtered response");
    assert!(
        v.get("walletDistributions").is_none() && v.get("regionData").is_none(),
        "filtered response must omit walletDistributions/regionData, got: {v:?}"
    );
    let rewards = v
        .get("farmRewards")
        .and_then(|x| x.as_array())
        .expect("farmRewards array");
    assert_eq!(rewards.len(), 1, "expected only one farm after filtering");
    let fid = rewards[0]
        .get("farmId")
        .and_then(|x| x.as_str())
        .expect("farmId");
    assert_eq!(fid, "F1");
}
