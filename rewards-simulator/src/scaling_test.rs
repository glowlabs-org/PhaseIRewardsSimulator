use crate::competition_simulator::simulate;
use crate::models::{InputData, SolarFarm};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde_json::json;
use std::fs;
use tower::ServiceExt;

fn make_farm(id: &str, ia: u64, addr: &str, first_week: u64, weeks_alive: u64) -> SolarFarm {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    SolarFarm {
        farm_id: id.to_string(),
        asset_id: "usdg".to_string(),
        region_id: "utah".to_string(),
        weekly_impact_assets: BigInt::from_u64(ia).unwrap() * &scale,
        protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
        assets_required: BigInt::from_u64(100).unwrap() * &scale,
        rewards_address: Some(addr.to_string()),
        first_week,
        weeks_alive,
    }
}

fn build_input(ias: &[u64], first_week: u64) -> InputData {
    let addrs = [
        "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
        "0xa273164a466dbF9F0173996078fb382acC73F9E3",
        "0x0000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000002",
        "0x0000000000000000000000000000000000000003",
        "0x0000000000000000000000000000000000000004",
    ];
    let farms = ias
        .iter()
        .enumerate()
        .map(|(i, ia)| {
            make_farm(
                &format!("F{}", i + 1),
                *ia,
                addrs[i % addrs.len()],
                first_week,
                2,
            )
        })
        .collect::<Vec<_>>();
    InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    }
}

fn write_log(name: &str, input: &InputData, output: &serde_json::Value) {
    let _ = fs::create_dir_all("test-logs");
    let filename = format!("test-logs/out_{}.log", name);
    let input_json = serde_json::to_value(input).unwrap();
    let summary = json!({
        "test_name": name,
        "input": input_json,
        "output": output
    });
    let pretty = serde_json::to_string_pretty(&summary).unwrap();
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
                "netWeeklyImpactAssets": f.weekly_impact_assets.to_string(),
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

fn assert_week_order_matches_ia(input: &InputData, output: &crate::models::OutputData, week: u64) {
    let wk = output
        .weekly_rewards
        .iter()
        .find(|w| w.week_number == week)
        .expect("expected week present");
    use std::collections::HashMap;
    let mut ia_map: HashMap<String, BigInt> = HashMap::new();
    for f in &input.solar_farms {
        ia_map.insert(f.farm_id.clone(), f.weekly_impact_assets.clone());
    }
    let mut rewards = wk
        .per_farm_rewards
        .iter()
        .map(|r| (r.farm_id.clone(), r.amount.clone()))
        .collect::<Vec<_>>();
    rewards.sort_by(|a, b| a.1.cmp(&b.1));
    let mut ias = input
        .solar_farms
        .iter()
        .map(|f| (f.farm_id.clone(), f.weekly_impact_assets.clone()))
        .collect::<Vec<_>>();
    ias.sort_by(|a, b| a.1.cmp(&b.1));
    assert_eq!(rewards.first().unwrap().0, ias.first().unwrap().0);
    assert_eq!(rewards.last().unwrap().0, ias.last().unwrap().0);
    for i in 1..rewards.len() {
        let (prev_id, prev_amt) = &rewards[i - 1];
        let (cur_id, cur_amt) = &rewards[i];
        let prev_ia = &ia_map[prev_id];
        let cur_ia = &ia_map[cur_id];
        if prev_ia < cur_ia {
            assert!(prev_amt <= cur_amt);
        } else if prev_ia > cur_ia {
            assert!(prev_amt >= cur_ia);
        }
    }
}

#[test]
fn scaling_two_farms_same_competition() {
    let input = build_input(&[1, 3], 90);
    let input_for_log = input.clone();
    let out = simulate(input.clone()).expect("simulation ok");
    assert_eq!(out.weekly_rewards.len(), 2);
    assert_eq!(out.total_regions, 1);
    assert_week_order_matches_ia(&input_for_log, &out, 90);
    assert_week_order_matches_ia(&input_for_log, &out, 91);
    assert_both_endpoints_status(&input, StatusCode::OK);
    let out_json = serde_json::to_value(&out).unwrap();
    write_log("scaling_two_farms", &input_for_log, &out_json);
}

#[test]
fn scaling_three_farms_same_competition() {
    let input = build_input(&[1, 2, 7], 100);
    let input_for_log = input.clone();
    let out = simulate(input.clone()).expect("simulation ok");
    assert_eq!(out.weekly_rewards.len(), 2);
    assert_eq!(out.total_regions, 1);
    assert_week_order_matches_ia(&input_for_log, &out, 100);
    assert_week_order_matches_ia(&input_for_log, &out, 101);
    assert_both_endpoints_status(&input, StatusCode::OK);
    let out_json = serde_json::to_value(&out).unwrap();
    write_log("scaling_three_farms", &input_for_log, &out_json);
}

#[test]
fn scaling_four_farms_same_competition() {
    let input = build_input(&[1, 1, 2, 6], 110);
    let input_for_log = input.clone();
    let out = simulate(input.clone()).expect("simulation ok");
    assert_eq!(out.weekly_rewards.len(), 2);
    assert_eq!(out.total_regions, 1);
    assert_week_order_matches_ia(&input_for_log, &out, 110);
    assert_week_order_matches_ia(&input_for_log, &out, 111);
    assert_both_endpoints_status(&input, StatusCode::OK);
    let out_json = serde_json::to_value(&out).unwrap();
    write_log("scaling_four_farms", &input_for_log, &out_json);
}
