use crate::competition_simulator::simulate;
use crate::models::{InputData, SolarFarm};
use crate::preload::merge_v1_data;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde_json::json;
use std::collections::HashMap;
use std::fs;

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

#[test]
fn test_preload_v1_merges_data() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let v1_leftover = BigInt::from(1000u64) * &scale;

    let mut v1_leftovers = HashMap::new();
    v1_leftovers.insert(100, v1_leftover);
    let v1_data = InputData {
        cgp_leftovers: v1_leftovers,
        solar_farms: vec![SolarFarm {
            farm_id: "v1_farm_1".into(),
            asset_id: "usdg".into(),
            region_id: "cgp".into(),
            weekly_impact_assets: scale.clone(),
            protocol_deposit_value: BigInt::from(100u64) * &scale,
            assets_required: BigInt::from(100u64) * &scale,
            rewards_address: Some("0x1111111111111111111111111111111111111111".into()),
            first_week: 100,
            weeks_alive: 2,
        }],
    };

    let user_leftover = BigInt::from(500u64) * &scale;
    let mut user_leftovers = HashMap::new();
    user_leftovers.insert(100, user_leftover);
    let user_input = InputData {
        cgp_leftovers: user_leftovers,
        solar_farms: vec![SolarFarm {
            farm_id: "user_farm_1".into(),
            asset_id: "usdg".into(),
            region_id: "cgp".into(),
            weekly_impact_assets: scale.clone(),
            protocol_deposit_value: BigInt::from(100u64) * &scale,
            assets_required: BigInt::from(100u64) * &scale,
            rewards_address: Some("0x2222222222222222222222222222222222222222".into()),
            first_week: 100,
            weeks_alive: 2,
        }],
    };
    let merged_input = merge_v1_data(user_input, v1_data).unwrap();
    let output = simulate(merged_input.clone()).unwrap();

    assert_eq!(output.weekly_rewards[0].per_farm_rewards.len(), 2);

    let w100_rewards = &output.weekly_rewards[0];
    assert_eq!(w100_rewards.week_number, 100);

    let expected_reward = (BigInt::from(50u64) * &scale) + (BigInt::from(750u64) * &scale);

    for farm_reward in &w100_rewards.per_farm_rewards {
        assert_eq!(farm_reward.amount, expected_reward);
    }

    let out_json = serde_json::to_value(&output).unwrap();
    write_log("preload_v1_merges_data", &merged_input, &out_json);
}

#[test]
fn test_preload_v1_duplicate_farm_id_fails() {
    let v1_data = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "dup_id".into(),
            asset_id: "usdg".into(),
            region_id: "cgp".into(),
            weekly_impact_assets: BigInt::from(1u64),
            protocol_deposit_value: BigInt::from(100u64),
            assets_required: BigInt::from(100u64),
            rewards_address: Some("0x1111111111111111111111111111111111111111".into()),
            first_week: 1,
            weeks_alive: 2,
        }],
    };

    let user_input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "dup_id".into(),
            asset_id: "usdg".into(),
            region_id: "cgp".into(),
            weekly_impact_assets: BigInt::from(1u64),
            protocol_deposit_value: BigInt::from(100u64),
            assets_required: BigInt::from(100u64),
            rewards_address: Some("0x2222222222222222222222222222222222222222".into()),
            first_week: 100,
            weeks_alive: 2,
        }],
    };

    let result = merge_v1_data(user_input, v1_data);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("duplicate farm id from v1 data: dup_id"));
}

#[test]
fn test_preload_v1_without_user_input() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let v1_data = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "v1_only".into(),
            asset_id: "usdg".into(),
            region_id: "cgp".into(),
            weekly_impact_assets: scale.clone(),
            protocol_deposit_value: (BigInt::from(100u64) * &scale),
            assets_required: (BigInt::from(100u64) * &scale),
            rewards_address: Some("0x1111111111111111111111111111111111111111".into()),
            first_week: 1,
            weeks_alive: 2,
        }],
    };

    let user_input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![],
    };

    let merged_input = merge_v1_data(user_input, v1_data).unwrap();
    let output = simulate(merged_input.clone()).unwrap();

    assert_eq!(output.weekly_rewards[0].per_farm_rewards.len(), 1);
    assert_eq!(
        output.weekly_rewards[0].per_farm_rewards[0].farm_id,
        "v1_only"
    );

    let out_json = serde_json::to_value(&output).unwrap();
    write_log("preload_v1_without_user_input", &merged_input, &out_json);
}
