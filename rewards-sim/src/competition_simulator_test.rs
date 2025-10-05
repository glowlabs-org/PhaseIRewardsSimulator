use crate::competition_simulator::{
    perform_consistency_checks, simulate, FarmRewardOut, WalletDistribution, WalletTrace,
};
use crate::models::{InputData, RewardSplit, SolarFarm};
use crate::test_utils::assert_both_endpoints_status;
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, One};
use std::collections::{BTreeMap, HashMap};

#[test]
fn consistency_checks_detect_mismatches() {
    let scale = BigInt::from_u128(1_000_000_000_000_000_000).unwrap();
    let one_asset = scale.clone();
    let one_asset_plus_one = &one_asset + BigInt::from(1);

    let assets_earned_val: BigInt = &one_asset * 100;
    let glow_earned_val: BigInt = &one_asset * 500;

    let mut wd = vec![WalletDistribution {
        user_address: "0x123".to_string(),
        assets_earned: BTreeMap::from([("glw".to_string(), assets_earned_val.to_string())]),
        glow_inflation_earned: glow_earned_val.clone(),
        traces: vec![WalletTrace {
            farm_id: "F1".to_string(),
            asset: "glw".to_string(),
            inflation_reward_split_6_decimals: BigInt::from(1000000),
            deposit_reward_split_6_decimals: BigInt::from(1000000),
            amount: assets_earned_val.clone(),
            region_id: 1,
            glow_inflation_reward: glow_earned_val.clone(),
        }],
    }];

    let mut fr = vec![FarmRewardOut {
        id: "F1".to_string(),
        asset: "glw".to_string(),
        region_id: 1,
        asset_earned: assets_earned_val.clone(),
        glow_inflation_reward: glow_earned_val.clone(),
        protocol_deposit: BigInt::from(1000),
        expected_production: BigInt::from(10),
    }];

    let mut warnings = Vec::new();
    perform_consistency_checks(1, &wd, &fr, &mut warnings);
    assert!(warnings.is_empty(), "Should be consistent initially");

    // Test case 1: wallet earned vs trace amount mismatch
    let bad_val_str = (&assets_earned_val - &one_asset_plus_one).to_string();
    wd[0].assets_earned.insert("glw".to_string(), bad_val_str);
    perform_consistency_checks(1, &wd, &fr, &mut warnings);
    assert!(
        !warnings.is_empty(),
        "Expected at least one warning for wallet asset/trace mismatch"
    );
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("asset 'glw' amount mismatch")),
        "Expected a specific asset mismatch warning, got: {:?}",
        warnings
    );
    warnings.clear();
    wd[0]
        .assets_earned
        .insert("glw".to_string(), assets_earned_val.to_string()); // reset

    // Test case 2: wallet glow vs trace glow mismatch
    wd[0].glow_inflation_earned -= &one_asset_plus_one;
    perform_consistency_checks(1, &wd, &fr, &mut warnings);
    assert!(
        !warnings.is_empty(),
        "Expected at least one warning for wallet glow/trace mismatch"
    );
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("glow inflation mismatch")),
        "Expected a specific glow inflation mismatch warning, got: {:?}",
        warnings
    );
    warnings.clear();
    wd[0].glow_inflation_earned = glow_earned_val.clone(); // reset

    // Test case 3: total assets wallet vs farm mismatch
    fr[0].asset_earned -= &one_asset_plus_one;
    perform_consistency_checks(1, &wd, &fr, &mut warnings);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("Total asset 'glw' mismatch")),
        "Expected total asset mismatch warning, got: {:?}",
        warnings
    );
    warnings.clear();
    fr[0].asset_earned = assets_earned_val.clone(); // reset

    // Test case 4: total glow wallet vs farm mismatch
    fr[0].glow_inflation_reward -= &one_asset_plus_one;
    perform_consistency_checks(1, &wd, &fr, &mut warnings);
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("Total glow inflation mismatch")),
        "Expected total glow inflation mismatch warning, got: {:?}",
        warnings
    );
    warnings.clear();
}

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
                region_id: 1,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(100).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
                first_week: 50,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "F2".into(),
                asset_id: "usdg".into(),
                region_id: 1,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(100).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
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
                region_id: 1000,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
                first_week: 1,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "dup".into(),
                asset_id: "usdg".into(),
                region_id: 1000,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
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
                region_id: 1003,
                weekly_impact_assets: BigInt::from_u64(0).unwrap(),
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
                first_week: 1,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "nz1".into(),
                asset_id: "glw".into(),
                region_id: 1003,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
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
                region_id: 1,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(300).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
                first_week: 2,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "B".into(),
                asset_id: "usdg".into(),
                region_id: 2,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(200).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
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
            region_id: 1001,
            weekly_impact_assets: BigInt::one(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
            assets_required: BigInt::from_u64(10).unwrap() * &scale,
            reward_split: vec![RewardSplit {
                wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
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
            region_id: 1002,
            weekly_impact_assets: BigInt::one(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
            assets_required: BigInt::from_u64(10).unwrap() * &scale,
            reward_split: vec![RewardSplit {
                wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
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
                region_id: 1,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10000).unwrap() * &scale,
                assets_required: BigInt::from_u64(20000).unwrap() * &scale,
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
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(10000).unwrap() * &scale,
                assets_required: BigInt::from_u64(20000).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
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

#[test]
fn cgp_leftovers_bonus_applied_case_insensitive() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();

    let mut leftovers = HashMap::new();
    leftovers.insert(50_u64, BigInt::from_u64(200).unwrap() * &scale);

    let input = InputData {
        cgp_leftovers: leftovers,
        solar_farms: vec![
            SolarFarm {
                farm_id: "F1".into(),
                asset_id: "USDG".into(), // Uppercase
                region_id: 1,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(100).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
                first_week: 50,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "F2".into(),
                asset_id: "USDG".into(), // Uppercase
                region_id: 1,
                weekly_impact_assets: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(100).unwrap() * &scale,
                reward_split: vec![RewardSplit {
                    wallet_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                    glow_split_percent_6_decimals: BigInt::from(1_000_000),
                    deposit_split_percent_6_decimals: BigInt::from(1_000_000),
                }],
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
        // base reward is 50, bonus is 100. total 150.
        assert_eq!(r.amount, BigInt::from_u64(150).unwrap() * &scale);
    }

    assert_both_endpoints_status(&input, StatusCode::OK);
}
