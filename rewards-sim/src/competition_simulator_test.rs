use crate::competition_simulator::simulate;
use crate::models::{InputData, SolarFarm};
use crate::test_utils::assert_both_endpoints_status;
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, One};
use std::collections::HashMap;

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
                rewards_address: Some("0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into()),
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
                rewards_address: Some("0xa273164a466dbF9F0173996078fb382acC73F9E3".into()),
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
                rewards_address: Some("0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into()),
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
                rewards_address: Some("0xa273164a466dbF9F0173996078fb382acC73F9E3".into()),
                first_week: 1,
                weeks_alive: 2,
            },
        ],
    };
    assert!(simulate(input.clone()).is_err());
    assert_both_endpoints_status(&input, StatusCode::BAD_REQUEST);
}

#[test]
fn zero_impact_assets_allowed() {
    // Zero weekly impact assets are allowed at the farm level.
    // Ensure simulation succeeds if the bucket has non-zero total impact assets.
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "z0".into(),
                asset_id: "glw".into(),
                region_id: "reg".into(),
                weekly_impact_assets: BigInt::from_u64(0).unwrap(),
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                rewards_address: Some("0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into()),
                first_week: 1,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "nz1".into(),
                asset_id: "glw".into(),
                region_id: "reg".into(),
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                rewards_address: Some("0xa273164a466dbF9F0173996078fb382acC73F9E3".into()),
                first_week: 1,
                weeks_alive: 2,
            },
        ],
    };
    let res = simulate(input.clone());
    assert!(
        res.is_ok(),
        "simulation should accept zero netWeeklyImpactAssets"
    );
    assert_both_endpoints_status(&input, StatusCode::OK);
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
                rewards_address: Some("0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into()),
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
                rewards_address: Some("0xa273164a466dbF9F0173996078fb382acC73F9E3".into()),
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
            rewards_address: Some("0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into()),
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
            rewards_address: Some("0xa273164a466dbF9F0173996078fb382acC73F9E3".into()),
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
                rewards_address: Some("0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into()),
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
                rewards_address: Some("0xa273164a466dbF9F0173996078fb382acC73F9E3".into()),
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
