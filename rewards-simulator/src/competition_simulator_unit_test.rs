use crate::competition_simulator::simulate;
use crate::models::{is_valid_eth_address, InputData, SolarFarm};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, One};
use std::collections::HashMap;
use tower::ServiceExt;

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
            serde_json::json!({
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

    serde_json::json!({
        "cgpLeftovers": serde_json::Value::Object(cgp_leftovers),
        "solarFarms": farms
    })
}

fn assert_both_endpoints_status(input: &InputData, expected: StatusCode) {
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

#[test]
fn eth_address_validation() {
    assert!(is_valid_eth_address(
        "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D"
    ));
    assert!(!is_valid_eth_address("0x123"));
    assert!(!is_valid_eth_address(
        "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
    ));
    assert!(!is_valid_eth_address(
        "6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D"
    ));
}

#[test]
fn basic_build_and_simulate() {
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "A".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10000).unwrap(),
                assets_required: BigInt::from_u64(20000).unwrap(), // 2 per unit
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 10,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "B".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10000).unwrap(),
                assets_required: BigInt::from_u64(20000).unwrap(),
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 10,
                weeks_alive: 2,
            },
        ],
    };
    let out = simulate(input.clone()).expect("ok");
    assert_eq!(out.total_regions, 1);
    assert_eq!(out.regional_stats.len(), 1);
    assert!(!out.weekly_rewards.is_empty());
    let wk10 = out
        .weekly_rewards
        .iter()
        .find(|w| w.week_number == 10)
        .unwrap();
    for r in &wk10.per_farm_rewards {
        assert_eq!(r.amount, BigInt::from_u64(10000).unwrap());
    }

    // Endpoints should accept this input
    assert_both_endpoints_status(&input, StatusCode::OK);
}
