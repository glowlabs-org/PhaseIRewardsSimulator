use crate::competition_simulator::simulate;
use crate::models::{InputData, SolarFarm};
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
            rewards_address: Some("0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into()),
            reward_split: vec![],
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
            region_id: 123457,
            weekly_impact_assets: BigInt::from(1u32),
            protocol_deposit_value: BigInt::from(10u32),
            assets_required: BigInt::from(10u32),
            rewards_address: Some("0xa273164a466dbF9F0173996078fb382acC73F9E3".into()),
            reward_split: vec![],
            first_week: 1,
            weeks_alive: 4097,
        }],
    };
    assert!(simulate(input.clone()).is_err());
    assert_both_endpoints_status(&input, StatusCode::BAD_REQUEST);
}
