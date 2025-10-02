use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::json;
use std::fs;
use tower::ServiceExt;

use crate::models::{InputData, RewardSplit};

pub fn write_log(name: &str, input: &InputData, output: &serde_json::Value) {
    let _ = fs::create_dir_all("test-logs");
    let filename = format!("test-logs/out_{}.log", name);
    let input_json = serde_json::to_value(input).expect("input to json");
    let summary = json!({
        "test_name": name,
        "input": input_json,
        "output": output
    });
    let pretty = serde_json::to_string_pretty(&summary).expect("stringify");
    fs::write(filename, pretty).expect("write log file");
}

fn map_region_to_json(region_id: &str) -> serde_json::Value {
    match region_id.to_lowercase().as_str() {
        "cgp" => serde_json::Value::Number(1u64.into()),
        "utah" => serde_json::Value::Number(2u64.into()),
        // For other regions, keep string form (backward-compatible and sufficient for tests)
        _ => serde_json::Value::String(region_id.to_string()),
    }
}

pub fn to_api_json(input: &InputData) -> serde_json::Value {
    let cgp_leftovers = input
        .cgp_leftovers
        .iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.to_string())))
        .collect::<serde_json::Map<String, serde_json::Value>>();

    let farms = input
        .solar_farms
        .iter()
        .map(|f| {
            let reward_split_json = if !f.reward_split.is_empty() {
                let arr: Vec<serde_json::Value> = f
                    .reward_split
                    .iter()
                    .map(|rs: &RewardSplit| {
                        json!({
                            "walletAddress": rs.wallet_address,
                            "glowSplitPercent6Decimals": rs.glow_split_percent_6_decimals.to_string(),
                            "depositSplitPercent6Decimals": rs.deposit_split_percent_6_decimals.to_string(),
                        })
                    })
                    .collect();
                serde_json::Value::Array(arr)
            } else {
                serde_json::Value::Null
            };

            let mut obj = json!({
                "farmId": f.farm_id,
                "assetId": f.asset_id,
                "regionId": map_region_to_json(&f.region_id),
                "netWeeklyImpactAssets": f.weekly_impact_assets.to_string(),
                "protocolDepositValue": f.protocol_deposit_value.to_string(),
                "assetsRequired": f.assets_required.to_string(),
                "rewardsAddress": f.rewards_address,
                "firstWeek": f.first_week,
                "weeksAlive": f.weeks_alive
            });
            if !f.reward_split.is_empty() {
                if let Some(map) = obj.as_object_mut() {
                    map.insert("rewardSplit".to_string(), reward_split_json);
                }
            }
            obj
        })
        .collect::<Vec<_>>();

    json!({
        "cgpLeftovers": serde_json::Value::Object(cgp_leftovers),
        "solarFarms": farms
    })
}

pub fn assert_both_endpoints_status(input: &InputData, expected: StatusCode) {
    let app = crate::server::app();
    let body_json = to_api_json(input);
    let body = serde_json::to_vec(&body_json).expect("serialize body");
    let req_basic = Request::post("/api/rewards-simulator")
        .header("content-type", "application/json")
        .body(Body::from(body.clone()))
        .unwrap();
    let req_detailed = Request::post("/api/rewards-simulator-detailed")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (st_basic, st_det) = rt.block_on(async move {
        let res1 = app.clone().oneshot(req_basic).await.unwrap();
        let res2 = app.oneshot(req_detailed).await.unwrap();
        (res1.status(), res2.status())
    });
    assert_eq!(st_basic, expected, "basic endpoint status mismatch");
    assert_eq!(st_det, expected, "detailed endpoint status mismatch");
}

pub async fn post_and_read(
    app: Router,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, String) {
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
    let text = String::from_utf8_lossy(&bytes).to_string();
    (status, text)
}
