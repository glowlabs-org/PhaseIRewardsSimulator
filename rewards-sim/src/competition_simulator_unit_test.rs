use crate::competition_simulator::simulate;
use crate::models::{is_valid_eth_address, InputData, RewardSplit, SolarFarm};
use crate::test_utils::assert_both_endpoints_status;
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, One};
use std::collections::HashMap;

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
                region_id: 1,
                weekly_impact_assets: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10000).unwrap(),
                assets_required: BigInt::from_u64(20000).unwrap(), // 2 per unit
                reward_split: vec![RewardSplit {
                    wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
                first_week: 10,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "B".into(),
                asset_id: "glw".into(),
                region_id: 1,
                weekly_impact_assets: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10000).unwrap(),
                assets_required: BigInt::from_u64(20000).unwrap(),
                reward_split: vec![RewardSplit {
                    wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
                first_week: 10,
                weeks_alive: 2,
            },
        ],
        output_farms: None,
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
