use crate::competition_simulator::simulate_multi_asset;
use crate::errors::SimError;
use crate::models::{AssetRequirement, InputDataMultiAsset, MultiAssetSolarFarm, RewardSplit};
use num_bigint::BigInt;
use std::collections::HashMap;

fn scale_1e18() -> BigInt {
    BigInt::from(1_000_000_000_000_000_000u64)
}

fn scale_1e6() -> BigInt {
    BigInt::from(1_000_000u64)
}

fn default_split() -> Vec<RewardSplit> {
    vec![RewardSplit {
        wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".to_string(),
        glow_split_percent_6_decimals: BigInt::from(1_000_000),
        deposit_split_percent_6_decimals: BigInt::from(1_000_000),
    }]
}

#[test]
fn test_multi_asset_happy_path_two_assets() {
    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![MultiAssetSolarFarm {
            farm_id: "F1".to_string(),
            region_id: 1,
            net_weekly_impact_assets: BigInt::from(300) * scale_1e18(),
            total_protocol_deposit_value: BigInt::from(300) * scale_1e6(),
            first_week: 10,
            weeks_alive: 5,
            assets: vec![
                AssetRequirement {
                    asset_id: "GLW".to_string(),
                    assets_required: BigInt::from(100) * scale_1e18(),
                    assets_required_usdc: BigInt::from(100) * scale_1e6(),
                    quoted_by_gve_price_per_asset: scale_1e6(),
                    decimals: Some(18),
                },
                AssetRequirement {
                    asset_id: "USDG".to_string(),
                    assets_required: BigInt::from(200) * scale_1e6(),
                    assets_required_usdc: BigInt::from(200) * scale_1e6(),
                    quoted_by_gve_price_per_asset: scale_1e6(),
                    decimals: Some(6),
                },
            ],
            reward_split: default_split(),
        }],
        gctl_distribution: None,
        output_farms: None,
    };

    let (output, errors) = simulate_multi_asset(input, false).expect("simulation failed");
    assert!(errors.is_empty());

    let week_out = output.get("10").expect("week 10 output missing");
    assert_eq!(week_out.farm_rewards.len(), 1);
    let farm_reward = &week_out.farm_rewards[0];
    assert_eq!(farm_reward.farm_id, "F1");

    let glw_reward = farm_reward
        .assets
        .iter()
        .find(|a| a.asset_id == "GLW")
        .expect("GLW missing");
    assert_eq!(glw_reward.asset_earned, BigInt::from(20) * scale_1e18());

    let usdg_reward = farm_reward
        .assets
        .iter()
        .find(|a| a.asset_id == "USDG")
        .expect("USDG missing");
    assert_eq!(usdg_reward.asset_earned, BigInt::from(40) * scale_1e6());
}

#[test]
fn test_multi_asset_validation_invalid_asset_id() {
    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![MultiAssetSolarFarm {
            farm_id: "BadID".to_string(),
            region_id: 1,
            net_weekly_impact_assets: scale_1e18(),
            total_protocol_deposit_value: BigInt::from(100) * scale_1e6(),
            first_week: 10,
            weeks_alive: 5,
            assets: vec![AssetRequirement {
                asset_id: "BAD".to_string(),
                assets_required: BigInt::from(100) * scale_1e18(),
                assets_required_usdc: BigInt::from(100) * scale_1e6(),
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: None,
            }],
            reward_split: default_split(),
        }],
        gctl_distribution: None,
        output_farms: None,
    };
    match simulate_multi_asset(input, false) {
        Err(SimError::Validation(msg)) => {
            assert!(msg.contains("Invalid assetId"));
        }
        _ => panic!("Expected Validation error for invalid asset ID"),
    }
}

#[test]
fn test_multi_asset_validation_invalid_decimals() {
    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![MultiAssetSolarFarm {
            farm_id: "BadDec".to_string(),
            region_id: 1,
            net_weekly_impact_assets: scale_1e18(),
            total_protocol_deposit_value: BigInt::from(100) * scale_1e6(),
            first_week: 10,
            weeks_alive: 5,
            assets: vec![AssetRequirement {
                asset_id: "GLW".to_string(),
                assets_required: BigInt::from(100) * scale_1e18(),
                assets_required_usdc: BigInt::from(100) * scale_1e6(),
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(6), // Wrong! GLW should be 18
            }],
            reward_split: default_split(),
        }],
        gctl_distribution: None,
        output_farms: None,
    };
    match simulate_multi_asset(input, false) {
        Err(SimError::Validation(msg)) => {
            assert!(msg.contains("Invalid decimals"));
        }
        _ => panic!("Expected Validation error for invalid decimals"),
    }
}

#[test]
fn test_multi_asset_output_filtering() {
    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        gctl_distribution: None,
        output_farms: Some(vec!["F1".to_string()]),
        solar_farms: vec![
            MultiAssetSolarFarm {
                farm_id: "F1".to_string(),
                region_id: 1,
                net_weekly_impact_assets: scale_1e18(),
                total_protocol_deposit_value: BigInt::from(10) * scale_1e6(),
                first_week: 10,
                weeks_alive: 5,
                reward_split: default_split(),
                assets: vec![AssetRequirement {
                    asset_id: "USDG".to_string(),
                    assets_required: BigInt::from(10) * scale_1e6(),
                    assets_required_usdc: BigInt::from(10) * scale_1e6(),
                    quoted_by_gve_price_per_asset: scale_1e6(),
                    decimals: None,
                }],
            },
            MultiAssetSolarFarm {
                farm_id: "F2".to_string(),
                region_id: 1,
                net_weekly_impact_assets: scale_1e18(),
                total_protocol_deposit_value: BigInt::from(10) * scale_1e6(),
                first_week: 10,
                weeks_alive: 5,
                reward_split: default_split(),
                assets: vec![AssetRequirement {
                    asset_id: "USDG".to_string(),
                    assets_required: BigInt::from(10) * scale_1e6(),
                    assets_required_usdc: BigInt::from(10) * scale_1e6(),
                    quoted_by_gve_price_per_asset: scale_1e6(),
                    decimals: None,
                }],
            },
        ],
    };

    let (output, _) = simulate_multi_asset(input, false).unwrap();
    let w10 = output.get("10").unwrap();
    assert_eq!(w10.farm_rewards.len(), 1);
    assert_eq!(w10.farm_rewards[0].farm_id, "F1");
    assert!(w10.wallet_distributions.is_empty());
}

#[test]
fn test_multi_asset_multiple_farms_overlap() {
    // Tests two farms competing in the same asset pools within the same region
    let scale_18 = BigInt::from(1_000_000_000_000_000_000u64);
    let scale_6 = BigInt::from(1_000_000u64);

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        gctl_distribution: None,
        output_farms: None,
        solar_farms: vec![
            MultiAssetSolarFarm {
                farm_id: "F1".to_string(),
                region_id: 1,
                net_weekly_impact_assets: BigInt::from(200) * &scale_18,
                total_protocol_deposit_value: BigInt::from(200) * &scale_6,
                first_week: 10,
                weeks_alive: 5,
                reward_split: default_split(),
                assets: vec![
                    AssetRequirement {
                        asset_id: "GLW".to_string(),
                        assets_required: BigInt::from(1000) * &scale_18, // say price is 0.1, so $100
                        assets_required_usdc: BigInt::from(100) * &scale_6,
                        quoted_by_gve_price_per_asset: BigInt::from(100_000), // 0.1
                        decimals: None,
                    },
                    AssetRequirement {
                        asset_id: "USDG".to_string(),
                        assets_required: BigInt::from(100) * &scale_6, // price 1, so $100
                        assets_required_usdc: BigInt::from(100) * &scale_6,
                        quoted_by_gve_price_per_asset: scale_6.clone(),
                        decimals: None,
                    },
                ],
            },
            MultiAssetSolarFarm {
                farm_id: "F2".to_string(),
                region_id: 1,
                net_weekly_impact_assets: BigInt::from(200) * &scale_18,
                total_protocol_deposit_value: BigInt::from(200) * &scale_6,
                first_week: 10,
                weeks_alive: 5,
                reward_split: default_split(),
                assets: vec![
                    AssetRequirement {
                        asset_id: "GLW".to_string(),
                        assets_required: BigInt::from(1000) * &scale_18,
                        assets_required_usdc: BigInt::from(100) * &scale_6,
                        quoted_by_gve_price_per_asset: BigInt::from(100_000),
                        decimals: None,
                    },
                    AssetRequirement {
                        asset_id: "SGCTL".to_string(),
                        assets_required: BigInt::from(50) * &scale_6, // price 2, so $100
                        assets_required_usdc: BigInt::from(100) * &scale_6,
                        quoted_by_gve_price_per_asset: BigInt::from(2_000_000),
                        decimals: None,
                    },
                ],
            },
        ],
    };

    let (output, _) = simulate_multi_asset(input, false).expect("simulation success");
    let w10 = output.get("10").expect("week 10");

    assert_eq!(w10.farm_rewards.len(), 2);

    // Check F1
    let f1 = w10.farm_rewards.iter().find(|f| f.farm_id == "F1").unwrap();
    // F1 should earn from GLW and USDG
    // GLW comp: F1 provides 50% deposits, 50% impact. Earns ~50% of inflation (if any) and preserves capital.
    // USDG comp: F1 is alone.

    // Check assets earned
    // In GLW comp (Region 1): Total Dep $200. F1 $100.
    // F1 recovers $20 (per week, since weeks_alive=5).
    // F1 recovers 20 / 0.1 = 200 GLW.
    let glw_r = f1.assets.iter().find(|a| a.asset_id == "GLW").unwrap();
    // 200 * 1e18
    assert_eq!(glw_r.asset_earned, BigInt::from(200) * &scale_18);

    // In USDG comp: Total Dep $100. F1 $100.
    // F1 recovers $20.
    // F1 recovers 20 / 1 = 20 USDG.
    let usdg_r = f1.assets.iter().find(|a| a.asset_id == "USDG").unwrap();
    // 20 * 1e6
    assert_eq!(usdg_r.asset_earned, BigInt::from(20) * &scale_6);

    // Check F2
    let f2 = w10.farm_rewards.iter().find(|f| f.farm_id == "F2").unwrap();
    // GLW comp: F2 recovers $20. 200 GLW.
    let glw_r2 = f2.assets.iter().find(|a| a.asset_id == "GLW").unwrap();
    assert_eq!(glw_r2.asset_earned, BigInt::from(200) * &scale_18);

    // SGCTL comp: Alone.
    // F2 recovers $20.
    // Price 2. So 10 SGCTL.
    let sgctl_r = f2.assets.iter().find(|a| a.asset_id == "SGCTL").unwrap();
    // 10 * 1e6
    assert_eq!(sgctl_r.asset_earned, BigInt::from(10) * &scale_6);
}
