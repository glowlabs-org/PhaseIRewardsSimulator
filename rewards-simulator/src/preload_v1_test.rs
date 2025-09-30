use crate::models::{InputData, SolarFarm};
use crate::server::app;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use once_cell::sync::Lazy;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use tower::ServiceExt;

static V1_DATA_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn write_log(name: &str, input: &serde_json::Value, output: &serde_json::Value) {
    let _ = fs::create_dir_all("test-logs");
    let filename = format!("test-logs/out_{}.log", name);
    let summary = json!({
        "test_name": name,
        "input": input,
        "output": output
    });
    let pretty = serde_json::to_string_pretty(&summary).expect("stringify");
    fs::write(filename, pretty).expect("write log file");
}

fn to_api_json(input: &InputData) -> serde_json::Value {
    let cgp_leftovers = input
        .cgp_leftovers
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
        .collect::<serde_json::Map<String, serde_json::Value>>();

    let farms = input
        .solar_farms
        .iter()
        .map(|f| {
            json!({
                "farmId": f.farm_id,
                "assetId": f.asset_id,
                "regionId": f.region_id,
                "weeklyCarbonCredits": f.weekly_carbon_credits.to_string(),
                "protocolDepositValue": f.protocol_deposit_value.to_string(),
                "assetsRequired": f.assets_required.to_string(),
                "rewardsAddress": f.rewards_address,
                "firstWeek": f.first_week,
                "weeksAlive": f.weeks_alive
            })
        })
        .collect::<Vec<_>>();

    json!({
        "cgpLeftovers": serde_json::Value::Object(cgp_leftovers),
        "solarFarms": farms
    })
}

async fn post_json(path: &str, body: serde_json::Value) -> (StatusCode, String) {
    let app = app();
    let res = app
        .oneshot(
            Request::post(path)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&bytes).to_string();
    (status, body_str)
}

fn cleanup_v1_data() {
    let _ = fs::remove_file("v1-data.json");
}

#[tokio::test]
async fn test_preload_v1_merges_data() {
    let _guard = V1_DATA_MUTEX.lock().unwrap();
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let v1_leftover = BigInt::from(1000u64) * &scale;
    let v1_data = json!({
      "cgpLeftovers": {
        "100": v1_leftover.to_string(),
      },
      "solarFarms": [
        {
          "farmId": "v1_farm_1",
          "assetId": "usdg",
          "regionId": "cgp",
          "weeklyCarbonCredits": scale.to_string(),
          "protocolDepositValue": (BigInt::from(100u64) * &scale).to_string(),
          "assetsRequired": (BigInt::from(100u64) * &scale).to_string(),
          "rewardsAddress": "0x1111111111111111111111111111111111111111",
          "firstWeek": 100,
          "weeksAlive": 2
        }
      ]
    });
    fs::write("v1-data.json", serde_json::to_string(&v1_data).unwrap()).unwrap();

    let user_leftover = BigInt::from(500u64) * &scale;
    let mut user_leftovers = HashMap::new();
    user_leftovers.insert(100, user_leftover);

    let user_input = InputData {
        cgp_leftovers: user_leftovers,
        solar_farms: vec![SolarFarm {
            farm_id: "user_farm_1".into(),
            asset_id: "usdg".into(),
            region_id: "cgp".into(),
            weekly_carbon_credits: scale.clone(),
            protocol_deposit_value: BigInt::from(100u64) * &scale,
            assets_required: BigInt::from(100u64) * &scale,
            rewards_address: "0x2222222222222222222222222222222222222222".into(),
            first_week: 100,
            weeks_alive: 2,
        }],
    };
    let body_json = to_api_json(&user_input);

    let (status, body) = post_json(
        "/api/rewards-simulator?preloadGlowV1=true",
        body_json.clone(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let output: crate::models::OutputData = serde_json::from_str(&body).unwrap();

    assert_eq!(output.weekly_rewards[0].per_farm_rewards.len(), 2);

    let w100_rewards = &output.weekly_rewards[0];
    assert_eq!(w100_rewards.week_number, 100);

    let expected_reward = (BigInt::from(50u64) * &scale) + (BigInt::from(750u64) * &scale);

    for farm_reward in &w100_rewards.per_farm_rewards {
        assert_eq!(farm_reward.amount, expected_reward);
    }

    let out_json = serde_json::from_str(&body).unwrap();
    write_log("preload_v1_merges_data", &body_json, &out_json);

    cleanup_v1_data();
}

#[tokio::test]
async fn test_preload_v1_duplicate_farm_id_fails() {
    let _guard = V1_DATA_MUTEX.lock().unwrap();
    let v1_data = json!({
      "cgpLeftovers": {},
      "solarFarms": [{"farmId": "dup_id", "assetId": "usdg", "regionId": "cgp", "weeklyCarbonCredits": "1", "protocolDepositValue": "100", "assetsRequired": "100", "rewardsAddress": "0x1111111111111111111111111111111111111111", "firstWeek": 1, "weeksAlive": 2}]
    });
    fs::write("v1-data.json", serde_json::to_string(&v1_data).unwrap()).unwrap();

    let user_input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "dup_id".into(),
            asset_id: "usdg".into(),
            region_id: "cgp".into(),
            weekly_carbon_credits: BigInt::from(1u64),
            protocol_deposit_value: BigInt::from(100u64),
            assets_required: BigInt::from(100u64),
            rewards_address: "0x2222222222222222222222222222222222222222".into(),
            first_week: 100,
            weeks_alive: 2,
        }],
    };
    let body_json = to_api_json(&user_input);

    let (status, body) = post_json("/api/rewards-simulator?preloadGlowV1=true", body_json).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("duplicate farm id from v1 data: dup_id"));

    cleanup_v1_data();
}

#[tokio::test]
async fn test_preload_v1_without_user_input() {
    let _guard = V1_DATA_MUTEX.lock().unwrap();
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let v1_data = json!({
      "cgpLeftovers": {},
      "solarFarms": [{
          "farmId": "v1_only",
          "assetId": "usdg", "regionId": "cgp",
          "weeklyCarbonCredits": scale.to_string(),
          "protocolDepositValue": (BigInt::from(100u64) * &scale).to_string(),
          "assetsRequired": (BigInt::from(100u64) * &scale).to_string(),
          "rewardsAddress": "0x1111111111111111111111111111111111111111",
          "firstWeek": 1, "weeksAlive": 2
      }]
    });
    fs::write("v1-data.json", serde_json::to_string(&v1_data).unwrap()).unwrap();

    let user_input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![],
    };
    let body_json = to_api_json(&user_input);

    let (status, body) = post_json(
        "/api/rewards-simulator?preloadGlowV1=true",
        body_json.clone(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let output: crate::models::OutputData = serde_json::from_str(&body).unwrap();
    assert_eq!(output.weekly_rewards[0].per_farm_rewards.len(), 1);
    assert_eq!(
        output.weekly_rewards[0].per_farm_rewards[0].farm_id,
        "v1_only"
    );

    let out_json = serde_json::from_str(&body).unwrap();
    write_log("preload_v1_without_user_input", &body_json, &out_json);

    cleanup_v1_data();
}
