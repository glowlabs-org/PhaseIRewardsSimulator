use crate::models::{InputData, SolarFarm};
use crate::simulator::simulate;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde_json::json;
use std::fs;

fn make_farm(id: &str, cc: u64, addr: &str, first_week: u64, weeks_alive: u64) -> SolarFarm {
    SolarFarm {
        farm_id: id.to_string(),
        asset_id: "usdg".to_string(),
        region_id: "utah".to_string(),
        weekly_carbon_credits: BigInt::from_u64(cc).unwrap(),
        protocol_deposit_value: BigInt::from_u64(100).unwrap(),
        assets_required: BigInt::from_u64(100).unwrap(),
        rewards_address: addr.to_string(),
        first_week,
        weeks_alive,
    }
}

fn build_input(ccs: &[u64], first_week: u64) -> InputData {
    let addrs = [
        "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
        "0xa273164a466dbF9F0173996078fb382acC73F9E3",
        "0x0000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000002",
        "0x0000000000000000000000000000000000000003",
        "0x0000000000000000000000000000000000000004",
    ];
    let farms = ccs
        .iter()
        .enumerate()
        .map(|(i, cc)| {
            make_farm(
                &format!("F{}", i + 1),
                *cc,
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

fn assert_week_order_matches_cc(input: &InputData, output: &crate::models::OutputData, week: u64) {
    let wk = output
        .weekly_rewards
        .iter()
        .find(|w| w.week_number == week)
        .expect("expected week present");
    use std::collections::HashMap;
    let mut cc_map: HashMap<String, BigInt> = HashMap::new();
    for f in &input.solar_farms {
        cc_map.insert(f.farm_id.clone(), f.weekly_carbon_credits.clone());
    }
    let mut rewards = wk
        .per_farm_rewards
        .iter()
        .map(|r| (r.farm_id.clone(), r.amount.clone()))
        .collect::<Vec<_>>();
    rewards.sort_by(|a, b| a.1.cmp(&b.1));
    let mut ccs = input
        .solar_farms
        .iter()
        .map(|f| (f.farm_id.clone(), f.weekly_carbon_credits.clone()))
        .collect::<Vec<_>>();
    ccs.sort_by(|a, b| a.1.cmp(&b.1));
    assert_eq!(rewards.first().unwrap().0, ccs.first().unwrap().0);
    assert_eq!(rewards.last().unwrap().0, ccs.last().unwrap().0);
    for i in 1..rewards.len() {
        let (prev_id, prev_amt) = &rewards[i - 1];
        let (cur_id, cur_amt) = &rewards[i];
        let prev_cc = &cc_map[prev_id];
        let cur_cc = &cc_map[cur_id];
        if prev_cc < cur_cc {
            assert!(prev_amt <= cur_amt);
        } else if prev_cc > cur_cc {
            assert!(prev_amt >= cur_amt);
        }
    }
}

#[test]
fn scaling_two_farms_same_competition() {
    let input = build_input(&[1, 3], 90);
    let input_for_log = input.clone();
    let out = simulate(input).expect("simulation ok");
    assert_eq!(out.weekly_rewards.len(), 2);
    assert_eq!(out.total_regions, 1);
    assert_week_order_matches_cc(&input_for_log, &out, 90);
    assert_week_order_matches_cc(&input_for_log, &out, 91);
    let out_json = serde_json::to_value(&out).unwrap();
    write_log("scaling_two_farms", &input_for_log, &out_json);
}

#[test]
fn scaling_three_farms_same_competition() {
    let input = build_input(&[1, 2, 7], 100);
    let input_for_log = input.clone();
    let out = simulate(input).expect("simulation ok");
    assert_eq!(out.weekly_rewards.len(), 2);
    assert_eq!(out.total_regions, 1);
    assert_week_order_matches_cc(&input_for_log, &out, 100);
    assert_week_order_matches_cc(&input_for_log, &out, 101);
    let out_json = serde_json::to_value(&out).unwrap();
    write_log("scaling_three_farms", &input_for_log, &out_json);
}

#[test]
fn scaling_four_farms_same_competition() {
    let input = build_input(&[1, 1, 2, 6], 110);
    let input_for_log = input.clone();
    let out = simulate(input).expect("simulation ok");
    assert_eq!(out.weekly_rewards.len(), 2);
    assert_eq!(out.total_regions, 1);
    assert_week_order_matches_cc(&input_for_log, &out, 110);
    assert_week_order_matches_cc(&input_for_log, &out, 111);
    let out_json = serde_json::to_value(&out).unwrap();
    write_log("scaling_four_farms", &input_for_log, &out_json);
}
