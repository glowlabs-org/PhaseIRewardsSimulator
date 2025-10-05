use crate::server::app;
use crate::test_utils::post_and_read;
use axum::http::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn output_farms_restricts_output() {
    let app = app();
    let body = serde_json::json!({
      "solarFarms": [
        {
          "farmId": "F1",
          "assetId": "usdg",
          "regionId": 1,
          "netWeeklyImpactAssets": "1000000000000000000",
          "protocolDepositValue": "100000000",
          "assetsRequired": "100000000",
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 1,
          "weeksAlive": 2
        },
        {
          "farmId": "F2",
          "assetId": "usdg",
          "regionId": 1,
          "netWeeklyImpactAssets": "1000000000000000000",
          "protocolDepositValue": "100000000",
          "assetsRequired": "100000000",
          "rewardSplit": [{
              "walletAddress": "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 1,
          "weeksAlive": 2
        }
      ],
      "outputFarms": ["F1"]
    });

    let (status, text) = post_and_read(app, "/api/rewards-simulator", body).await;
    assert_eq!(status, StatusCode::OK);

    let v: Value = serde_json::from_str(&text).unwrap();
    let week1 = v.get("1").unwrap();

    assert!(week1.get("walletDistributions").is_none());
    assert!(week1.get("regionData").is_none());
    assert!(week1.get("farmRewards").is_some());
    assert!(week1.get("warnings").is_some());

    let farm_rewards = week1.get("farmRewards").unwrap().as_array().unwrap();
    assert_eq!(farm_rewards.len(), 1);
    assert_eq!(farm_rewards[0].get("id").unwrap().as_str().unwrap(), "F1");
}

#[tokio::test]
async fn output_farms_empty_list() {
    let app = app();
    let body = serde_json::json!({
      "solarFarms": [
        {
          "farmId": "F1",
          "assetId": "usdg",
          "regionId": 1,
          "netWeeklyImpactAssets": "1000000000000000000",
          "protocolDepositValue": "100000000",
          "assetsRequired": "100000000",
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 1,
          "weeksAlive": 2
        }
      ],
      "outputFarms": []
    });

    let (status, text) = post_and_read(app, "/api/rewards-simulator", body).await;
    assert_eq!(status, StatusCode::OK);

    let v: Value = serde_json::from_str(&text).unwrap();
    let week1 = v.get("1").unwrap();
    let farm_rewards = week1.get("farmRewards").unwrap().as_array().unwrap();
    assert_eq!(farm_rewards.len(), 0);
}

#[tokio::test]
async fn output_farms_nonexistent_id() {
    let app = app();
    let body = serde_json::json!({
      "solarFarms": [
        {
          "farmId": "F1",
          "assetId": "usdg",
          "regionId": 1,
          "netWeeklyImpactAssets": "1000000000000000000",
          "protocolDepositValue": "100000000",
          "assetsRequired": "100000000",
          "rewardSplit": [{
              "walletAddress": "0xa273164a466dbF9F0173996078fb382acC73F9E3",
              "glowSplitPercent6Decimals": "1000000",
              "depositSplitPercent6Decimals": "1000000"
          }],
          "firstWeek": 1,
          "weeksAlive": 2
        }
      ],
      "outputFarms": ["nonexistent"]
    });

    let (status, text) = post_and_read(app, "/api/rewards-simulator", body).await;
    assert_eq!(status, StatusCode::OK);

    let v: Value = serde_json::from_str(&text).unwrap();
    let week1 = v.get("1").unwrap();
    let farm_rewards = week1.get("farmRewards").unwrap().as_array().unwrap();
    assert_eq!(farm_rewards.len(), 0);
}
