use crate::competition_simulator::simulate;
use crate::models::{InputData, RewardSplit, SolarFarm};
use crate::test_utils::assert_both_endpoints_status;
use axum::http::StatusCode;
use num_bigint::BigInt;

#[test]
fn weeks_alive_upper_bound_inclusive_ok() {
    // Minimal values to keep runtime low while exercising 4096 weeks
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: vec![SolarFarm {
            farm_id: "WMAX".into(),
            asset_id: "glw".into(),
            region_id: 123457,
            weekly_impact_assets: BigInt::from(1u32),
            protocol_deposit_value: BigInt::from(4096u32),
            assets_required: BigInt::from(4096u32),
            reward_split: vec![RewardSplit {
                wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
            first_week: 1,
            weeks_alive: 4096,
        }],
        output_farms: None,
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
            region_id: 123457,
            weekly_impact_assets: BigInt::from(1u32),
            protocol_deposit_value: BigInt::from(10u32),
            assets_required: BigInt::from(10u32),
            reward_split: vec![RewardSplit {
                wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
            first_week: 1,
            weeks_alive: 4097,
        }],
        output_farms: None,
    };
    assert!(simulate(input.clone()).is_err());
    assert_both_endpoints_status(&input, StatusCode::BAD_REQUEST);
}
