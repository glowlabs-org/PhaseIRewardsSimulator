use crate::competition_simulator::{
    build_public_output_from_detailed, simulate, simulate_with_diagnostics,
};
use crate::models::{InputData, SolarFarm};
use crate::preload::merge_v1_data;
use crate::test_utils::write_log;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use std::collections::HashMap;

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
            region_id: 1,
            weekly_impact_assets: scale.clone(),
            protocol_deposit_value: BigInt::from(100u64) * &scale,
            assets_required: BigInt::from(100u64) * &scale,
            rewards_address: Some("0x1111111111111111111111111111111111111111".into()),
            reward_split: vec![],
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
            region_id: 1,
            weekly_impact_assets: scale.clone(),
            protocol_deposit_value: BigInt::from(100u64) * &scale,
            assets_required: BigInt::from(100u64) * &scale,
            rewards_address: Some("0x2222222222222222222222222222222222222222".into()),
            reward_split: vec![],
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
            region_id: 1,
            weekly_impact_assets: BigInt::from(1u64),
            protocol_deposit_value: BigInt::from(100u64),
            assets_required: BigInt::from(100u64),
            rewards_address: Some("0x1111111111111111111111111111111111111111".into()),
            reward_split: vec![],
            first_week: 1,
            weeks_alive: 2,
        }],
    };

    let user_input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "dup_id".into(),
            asset_id: "usdg".into(),
            region_id: 1,
            weekly_impact_assets: BigInt::from(1u64),
            protocol_deposit_value: BigInt::from(100u64),
            assets_required: BigInt::from(100u64),
            rewards_address: Some("0x2222222222222222222222222222222222222222".into()),
            reward_split: vec![],
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
            region_id: 1,
            weekly_impact_assets: scale.clone(),
            protocol_deposit_value: (BigInt::from(100u64) * &scale),
            assets_required: (BigInt::from(100u64) * &scale),
            rewards_address: Some("0x1111111111111111111111111111111111111111".into()),
            reward_split: vec![],
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

#[test]
fn test_preload_v1_public_output_includes_all_farms() {
    // Simulates the API flow where user farms are merged with V1 farms
    // and then the public-facing output is generated. This ensures that
    // user-provided farms are not dropped during output generation.

    // 1. User input farm
    let user_farm = SolarFarm {
        farm_id: "user_farm_new".into(),
        asset_id: "usdg".into(),
        region_id: 1,
        weekly_impact_assets: BigInt::from(1_000_000_000_000_000_000u128),
        protocol_deposit_value: BigInt::from(100_000_000u64),
        assets_required: BigInt::from(100_000_000u64),
        rewards_address: Some("0x1234567890123456789012345678901234567890".into()),
        reward_split: vec![],
        first_week: 96,
        weeks_alive: 2,
    };
    let user_input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![user_farm],
    };

    // 2. V1 data farm
    let v1_farm = SolarFarm {
        farm_id: "v1_farm_new".into(),
        asset_id: "usdg".into(),
        region_id: 1,
        weekly_impact_assets: BigInt::from(1_000_000_000_000_000_000u128),
        protocol_deposit_value: BigInt::from(100_000_000u64),
        assets_required: BigInt::from(100_000_000u64),
        rewards_address: Some("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".into()),
        reward_split: vec![],
        first_week: 96,
        weeks_alive: 2,
    };
    let v1_data = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![v1_farm],
    };

    // 3. Merge
    let merged_input = merge_v1_data(user_input, v1_data).unwrap();
    assert_eq!(merged_input.solar_farms.len(), 2);

    // 4. Simulate
    let diag = simulate_with_diagnostics(merged_input).unwrap();

    // 5. Build public output
    let public_output = build_public_output_from_detailed(&diag.competitions, &diag.errors);

    // 6. Assert
    assert!(
        !public_output.is_empty(),
        "public output should not be empty"
    );
    let week96_output = public_output.get("96").expect("week 96 should exist");

    let farm_ids: Vec<_> = week96_output
        .farm_rewards
        .iter()
        .map(|f| f.id.clone())
        .collect();

    assert_eq!(
        week96_output.farm_rewards.len(),
        2,
        "should have 2 farms in week 96 rewards"
    );
    assert!(
        farm_ids.contains(&"user_farm_new".to_string()),
        "user farm should be in output"
    );
    assert!(
        farm_ids.contains(&"v1_farm_new".to_string()),
        "v1 farm should be in output"
    );
}
