use crate::competition_simulator::simulate;
use crate::models::{InputData, SolarFarm};
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
                "netWeeklyImpactAssets": f.weekly_impact_assets.to_string(),
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
fn cgp_leftovers_bonus_applied() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();

    let mut leftovers = HashMap::new();
    leftovers.insert(50_u64, BigInt::from_u64(200).unwrap() * &scale);

    let input = InputData {
        cgp_leftovers: leftovers,
        solar_farms: vec![
            SolarFarm {
                farm_id: "F1".into(),
                asset_id: "usdg".into(),
                region_id: "cgp".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(100).unwrap() * &scale,
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 50,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "F2".into(),
                asset_id: "usdg".into(),
                region_id: "cgp".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(100).unwrap() * &scale,
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 50,
                weeks_alive: 2,
            },
        ],
    };

    let out = simulate(input.clone()).expect("ok");
    let wk = out
        .weekly_rewards
        .iter()
        .find(|w| w.week_number == 50)
        .unwrap();
    for r in &wk.per_farm_rewards {
        assert_eq!(r.amount, BigInt::from_u64(150).unwrap() * &scale);
    }

    assert_both_endpoints_status(&input, StatusCode::OK);
}

#[test]
fn duplicate_farm_id_rejected() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "dup".into(),
                asset_id: "usdg".into(),
                region_id: "x".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 1,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "dup".into(),
                asset_id: "usdg".into(),
                region_id: "x".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 1,
                weeks_alive: 2,
            },
        ],
    };
    assert!(simulate(input.clone()).is_err());
    assert_both_endpoints_status(&input, StatusCode::BAD_REQUEST);
}

#[test]
fn zero_impact_assets_rejected() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "z".into(),
            asset_id: "a".into(),
            region_id: "r".into(),
            weekly_impact_assets: BigInt::from_u64(0).unwrap(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
            assets_required: BigInt::from_u64(10).unwrap() * &scale,
            rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
            first_week: 1,
            weeks_alive: 2,
        }],
    };
    assert!(simulate(input.clone()).is_err());
    assert_both_endpoints_status(&input, StatusCode::BAD_REQUEST);
}

#[test]
fn happy_path_multiple_regions() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "A".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(300).unwrap() * &scale,
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 2,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "B".into(),
                asset_id: "usdg".into(),
                region_id: "utah".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(200).unwrap() * &scale,
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 2,
                weeks_alive: 2,
            },
        ],
    };
    let out = simulate(input.clone()).expect("ok");
    assert_eq!(out.total_regions, 2);
    let wk = &out.weekly_rewards[0];
    assert_eq!(wk.week_number, 2);
    let r_a = wk
        .per_farm_rewards
        .iter()
        .find(|r| r.farm_id == "A")
        .unwrap();
    assert_eq!(r_a.amount, BigInt::from_u64(150).unwrap() * &scale);
    let r_b = wk
        .per_farm_rewards
        .iter()
        .find(|r| r.farm_id == "B")
        .unwrap();
    assert_eq!(r_b.amount, BigInt::from_u64(100).unwrap() * &scale);
    assert_both_endpoints_status(&input, StatusCode::OK);
}

#[test]
fn weeks_alive_minimum_enforced() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "min1".into(),
            asset_id: "x".into(),
            region_id: "y".into(),
            weekly_impact_assets: BigInt::one(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
            assets_required: BigInt::from_u64(10).unwrap() * &scale,
            rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
            first_week: 10,
            weeks_alive: 1,
        }],
    };
    assert!(simulate(input.clone()).is_err());
    assert_both_endpoints_status(&input, StatusCode::BAD_REQUEST);
}

#[test]
fn weeks_alive_equal_two_allowed() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "ok2".into(),
            asset_id: "usdg".into(),
            region_id: "ok".into(),
            weekly_impact_assets: BigInt::one(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
            assets_required: BigInt::from_u64(10).unwrap() * &scale,
            rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
            first_week: 5,
            weeks_alive: 2,
        }],
    };
    assert!(simulate(input.clone()).is_ok());
    assert_both_endpoints_status(&input, StatusCode::OK);
}

#[test]
fn basic_build_and_simulate() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "A".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10000).unwrap() * &scale,
                assets_required: BigInt::from_u64(20000).unwrap() * &scale,
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 10,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "B".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10000).unwrap() * &scale,
                assets_required: BigInt::from_u64(20000).unwrap() * &scale,
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
        assert_eq!(r.amount, BigInt::from_u64(10000).unwrap() * &scale);
    }
    assert_both_endpoints_status(&input, StatusCode::OK);
}
