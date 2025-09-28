use crate::models::{InputData, SolarFarm};
use crate::simulator::simulate;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use num_bigint::BigInt;
use tower::ServiceExt;

fn assert_both_endpoints_status(input: &InputData, expected: StatusCode) {
    let app = crate::server::app();
    let body_json = serde_json::json!({
        "cgp_leftovers": {},
        "solar_farms": input.solar_farms.iter().map(|f| {
            serde_json::json!({
                "farm_id": f.farm_id,
                "asset_id": f.asset_id,
                "region_id": f.region_id,
                "weekly_carbon_credits": f.weekly_carbon_credits.to_string(),
                "protocol_deposit_value": f.protocol_deposit_value.to_string(),
                "assets_required": f.assets_required.to_string(),
                "rewards_address": f.rewards_address,
                "first_week": f.first_week,
                "weeks_alive": f.weeks_alive
            })
        }).collect::<Vec<_>>()
    });
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
fn weeks_alive_upper_bound_inclusive_ok() {
    // Minimal values to keep runtime low while exercising 4096 weeks
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: vec![SolarFarm {
            farm_id: "WMAX".into(),
            asset_id: "glw".into(),
            region_id: "sim".into(),
            weekly_carbon_credits: BigInt::from(1u32),
            protocol_deposit_value: BigInt::from(4096u32),
            assets_required: BigInt::from(4096u32),
            rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
            first_week: 1,
            weeks_alive: 4096,
        }],
    };

    let out = simulate(input.clone()).expect("simulation should succeed with weeks_alive=4096");
    assert!(!out.weekly_rewards.is_empty());
    assert_both_endpoints_status(&input, StatusCode::OK);
}

#[test]
fn weeks_alive_above_upper_bound_rejected() {
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: vec![SolarFarm {
            farm_id: "WTOO".into(),
            asset_id: "glw".into(),
            region_id: "sim".into(),
            weekly_carbon_credits: BigInt::from(1u32),
            protocol_deposit_value: BigInt::from(10u32),
            assets_required: BigInt::from(10u32),
            rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
            first_week: 1,
            weeks_alive: 4097,
        }],
    };
    assert!(simulate(input.clone()).is_err());
    assert_both_endpoints_status(&input, StatusCode::BAD_REQUEST);
}
