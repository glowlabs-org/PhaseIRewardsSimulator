use crate::models::{InputData, SolarFarm};
use crate::simulator::simulate_with_diagnostics;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde_json::json;
use std::fs;
use tower::ServiceExt;

fn write_log(name: &str, input: &InputData, output: &serde_json::Value) {
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
        })
        .collect::<Vec<_>>();

    json!({
        "cgp_leftovers": serde_json::Value::Object(cgp_leftovers),
        "solar_farms": farms
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
fn advanced_five_farms_single_region_specified() {
    // Scale factor to ensure deterministic integer math with low dust.
    let scale = BigInt::from_u64(1_000_000).unwrap();

    // Common protocol deposit and assets_required per spec (scaled).
    let proto = BigInt::from_u64(1_000).unwrap() * &scale;
    let assets_small = BigInt::from_u64(10).unwrap() * &scale;
    let assets_large = BigInt::from_u64(1_000).unwrap() * &scale;

    // Valid addresses.
    let addrs = [
        "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
        "0xa273164a466dbF9F0173996078fb382acC73F9E3",
        "0x0000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000002",
        "0x0000000000000000000000000000000000000003",
    ];

    // Farms with starts [1,2,3,4,5], weeks_alive=5, CCs [20,20,2,20,20].
    // All use same region/asset to create a single competition.
    let farms = vec![
        SolarFarm {
            farm_id: "F1".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_carbon_credits: BigInt::from_u64(20).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_small.clone(),
            rewards_address: addrs[0].into(),
            first_week: 1,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F2".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_carbon_credits: BigInt::from_u64(20).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_small.clone(),
            rewards_address: addrs[1].into(),
            first_week: 2,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F3".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_carbon_credits: BigInt::from_u64(2).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_large.clone(),
            rewards_address: addrs[2].into(),
            first_week: 3,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F4".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_carbon_credits: BigInt::from_u64(20).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_small.clone(),
            rewards_address: addrs[3].into(),
            first_week: 4,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F5".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_carbon_credits: BigInt::from_u64(20).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_small.clone(),
            rewards_address: addrs[4].into(),
            first_week: 5,
            weeks_alive: 5,
        },
    ];

    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };

    // Run with diagnostics so logs contain all consistency checks.
    let input_for_log = input.clone();
    let diag = simulate_with_diagnostics(input.clone()).expect("simulation with diagnostics ok");
    let out = diag.output;

    // Structural checks for a single competition spanning weeks 1..=9.
    assert_eq!(out.total_regions, 1, "expected single region");
    assert_eq!(
        out.weekly_rewards.len(),
        9,
        "expected contiguous weeks 1 through 9"
    );

    // Verify expected farm counts per active week window.
    let expected_counts = [
        (1_u64, 1_usize),
        (2, 2),
        (3, 3),
        (4, 4),
        (5, 5),
        (6, 4),
        (7, 3),
        (8, 2),
        (9, 1),
    ];
    for (week, expected_len) in expected_counts {
        let wk = out
            .weekly_rewards
            .iter()
            .find(|w| w.week_number == week)
            .unwrap_or_else(|| panic!("missing week {week}"));
        assert_eq!(
            wk.per_farm_rewards.len(),
            expected_len,
            "unexpected farm count for week {week}"
        );
    }

    // Ensure both API endpoints accept this input (200 OK).
    assert_both_endpoints_status(&input, StatusCode::OK);

    // Log input, output, and all consistency checks so humans can inspect pool dust and other checks.
    let out_json = serde_json::to_value(&out).expect("output to json");
    let log = json!({
        "diagnostics": {
            "consistency_errors": diag.errors,
        },
        "output": out_json
    });
    write_log(
        "advanced_five_farms_single_region_specified",
        &input_for_log,
        &log,
    );
}
