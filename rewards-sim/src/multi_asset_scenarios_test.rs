use crate::competition_simulator::simulate_multi_asset;
use crate::models::{AssetRequirement, InputDataMultiAsset, MultiAssetSolarFarm, RewardSplit};
use num_bigint::BigInt;
use num_traits::Signed;
use std::collections::HashMap;

// --- Helpers ---

fn scale_1e18() -> BigInt {
    BigInt::from(1_000_000_000_000_000_000u64)
}

fn scale_1e6() -> BigInt {
    BigInt::from(1_000_000u64)
}

fn make_split() -> Vec<RewardSplit> {
    vec![RewardSplit {
        wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".to_string(),
        glow_split_percent_6_decimals: BigInt::from(1_000_000),
        deposit_split_percent_6_decimals: BigInt::from(1_000_000),
    }]
}

#[test]
fn test_scenario_4_impact_distribution_accuracy() {
    // Scenario 4: Uneven split (60/25/15)
    // Farm has 10000 total impact.
    // Deposits: 60% GLW, 25% USDG, 15% SGCTL.
    let f1 = MultiAssetSolarFarm {
        farm_id: "Split".to_string(),
        region_id: 1,
        net_weekly_impact_assets: BigInt::from(10000) * scale_1e18(),
        total_protocol_deposit_value: BigInt::from(100) * scale_1e6(),
        first_week: 10,
        weeks_alive: 5,
        assets: vec![
            AssetRequirement {
                asset_id: "GLW".to_string(),
                assets_required: BigInt::from(60) * scale_1e18(),
                assets_required_usdc: BigInt::from(60) * scale_1e6(), // 60%
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(18),
            },
            AssetRequirement {
                asset_id: "USDG".to_string(),
                assets_required: BigInt::from(25) * scale_1e6(),
                assets_required_usdc: BigInt::from(25) * scale_1e6(), // 25%
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(6),
            },
            AssetRequirement {
                asset_id: "SGCTL".to_string(),
                assets_required: BigInt::from(15) * scale_1e6(),
                assets_required_usdc: BigInt::from(15) * scale_1e6(), // 15%
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(6),
            },
        ],
        reward_split: make_split(),
    };

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f1],
        gctl_distribution: None,
        output_farms: None,
    };

    let (output, _) = simulate_multi_asset(input, false).expect("simulation success");
    let week_out = output.get("10").unwrap();
    let fr = &week_out.farm_rewards[0];

    // Check expected production per asset in the output is not explicitly available per asset,
    // but expectedProduction is total.
    assert_eq!(fr.expected_production, BigInt::from(10000) * scale_1e18());

    // However, we can infer impact distribution by looking at rewards if they are the only farm.
    // If alone, they recover 100% of deposits in each pool.
    // GLW Pool: Deposit $60. Impact = 6000. Recovers $12/week.
    // USDG Pool: Deposit $25. Impact = 2500. Recovers $5/week.
    // SGCTL Pool: Deposit $15. Impact = 1500. Recovers $3/week.

    // Check assets earned (which equals recovery when alone and fully capitalized)
    let get_earned = |aid: &str| {
        fr.assets
            .iter()
            .find(|a| a.asset_id == aid)
            .unwrap()
            .asset_earned
            .clone()
    };

    // GLW: $12 recovered / $1 price = 12 GLW.
    assert_eq!(get_earned("GLW"), BigInt::from(12) * scale_1e18());

    // USDG: $5 recovered / $1 price = 5 USDG.
    assert_eq!(get_earned("USDG"), BigInt::from(5) * scale_1e6());

    // SGCTL: $3 recovered / $1 price = 3 SGCTL.
    assert_eq!(get_earned("SGCTL"), BigInt::from(3) * scale_1e6());
}

#[test]
fn test_scenario_5_cross_competition_isolation() {
    // Farm A: 100% GLW.
    // Farm B: 50% GLW / 50% USDG.
    // Verify Farm A only affects GLW.
    // Farm B's USDG is independent.

    let fa = MultiAssetSolarFarm {
        farm_id: "A".to_string(),
        region_id: 1,
        net_weekly_impact_assets: BigInt::from(100) * scale_1e18(),
        total_protocol_deposit_value: BigInt::from(100) * scale_1e6(),
        first_week: 10,
        weeks_alive: 5,
        assets: vec![AssetRequirement {
            asset_id: "GLW".to_string(),
            assets_required: BigInt::from(100) * scale_1e18(),
            assets_required_usdc: BigInt::from(100) * scale_1e6(),
            quoted_by_gve_price_per_asset: scale_1e6(),
            decimals: Some(18),
        }],
        reward_split: make_split(),
    };

    let fb = MultiAssetSolarFarm {
        farm_id: "B".to_string(),
        region_id: 1,
        net_weekly_impact_assets: BigInt::from(100) * scale_1e18(),
        total_protocol_deposit_value: BigInt::from(100) * scale_1e6(),
        first_week: 10,
        weeks_alive: 5,
        assets: vec![
            AssetRequirement {
                asset_id: "GLW".to_string(),
                assets_required: BigInt::from(50) * scale_1e18(),
                assets_required_usdc: BigInt::from(50) * scale_1e6(),
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(18),
            },
            AssetRequirement {
                asset_id: "USDG".to_string(),
                assets_required: BigInt::from(50) * scale_1e6(),
                assets_required_usdc: BigInt::from(50) * scale_1e6(),
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(6),
            },
        ],
        reward_split: make_split(),
    };

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![fa, fb],
        gctl_distribution: None,
        output_farms: None,
    };

    let (output, _) = simulate_multi_asset(input, false).unwrap();
    let week_out = output.get("10").unwrap();

    // Check B's USDG rewards. B is alone in USDG.
    // Impact = 50% of 100 = 50. Deposit = 50.
    // Should recover 100% of deposit => $10/week.
    // 10 USDG.
    let rb = week_out
        .farm_rewards
        .iter()
        .find(|r| r.farm_id == "B")
        .unwrap();
    let usdg_b = rb.assets.iter().find(|a| a.asset_id == "USDG").unwrap();
    assert_eq!(usdg_b.asset_earned, BigInt::from(10) * scale_1e6());

    // Check GLW competition. A vs B.
    // A: Impact 100. Deposit 100.
    // B: Impact 50. Deposit 50.
    // Ratios are identical (1:1). So both should recover 100%.
    // A recovers $20/week -> 20 GLW.
    // B recovers $10/week -> 10 GLW.
    let ra = week_out
        .farm_rewards
        .iter()
        .find(|r| r.farm_id == "A")
        .unwrap();
    let glw_a = ra.assets.iter().find(|a| a.asset_id == "GLW").unwrap();
    assert_eq!(glw_a.asset_earned, BigInt::from(20) * scale_1e18());

    let glw_b = rb.assets.iter().find(|a| a.asset_id == "GLW").unwrap();
    assert_eq!(glw_b.asset_earned, BigInt::from(10) * scale_1e18());
}

#[test]
fn test_scenario_6_edge_cases() {
    // Edge case: Very small deposits (Dust)
    let f_dust = MultiAssetSolarFarm {
        farm_id: "Dust".to_string(),
        region_id: 1,
        net_weekly_impact_assets: scale_1e18(),
        total_protocol_deposit_value: BigInt::from(2), // 2 micro-cents
        first_week: 1,
        weeks_alive: 2,
        assets: vec![AssetRequirement {
            asset_id: "USDG".to_string(),
            assets_required: BigInt::from(2),
            assets_required_usdc: BigInt::from(2),
            quoted_by_gve_price_per_asset: scale_1e6(),
            decimals: Some(6),
        }],
        reward_split: make_split(),
    };
    // Edge case: Max weeks alive (2^12)
    // Need to ensure weeks_alive fits logic.
    let f_long = MultiAssetSolarFarm {
        farm_id: "Long".to_string(),
        region_id: 2,
        net_weekly_impact_assets: scale_1e18(),
        total_protocol_deposit_value: BigInt::from(4096) * scale_1e6(),
        first_week: 1,
        weeks_alive: 4096, // Max
        assets: vec![AssetRequirement {
            asset_id: "GLW".to_string(),
            assets_required: BigInt::from(4096) * scale_1e18(),
            assets_required_usdc: BigInt::from(4096) * scale_1e6(),
            quoted_by_gve_price_per_asset: scale_1e6(),
            decimals: Some(18),
        }],
        reward_split: make_split(),
    };

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f_dust, f_long],
        gctl_distribution: None,
        output_farms: None,
    };

    let (output, errors) = simulate_multi_asset(input, false).expect("sim success");
    assert!(errors.is_empty());

    // Check Dust farm
    let w1 = output.get("1").unwrap();
    let r_dust = w1
        .farm_rewards
        .iter()
        .find(|r| r.farm_id == "Dust")
        .unwrap();
    // 2 units deposit / 2 weeks = 1 unit per week.
    // 100% recovery (alone).
    let usdg = r_dust.assets.iter().find(|a| a.asset_id == "USDG").unwrap();
    // 1e-6 USDG value. Price is 1. So 1e-12 USDG tokens?
    // Wait, price is 1e6 ($1).
    // Deposit value 1 (micro-cent). Recover 1 micro-cent worth.
    // 1 micro-cent / $1 = 0.000001 tokens = 1e-6 tokens.
    // But tokens are 6 decimals. So 1e-6 * 1e6 = 1 unit.
    assert_eq!(usdg.asset_earned, BigInt::from(1));

    // Check Long farm
    // 4096 weeks. Deposit 4096 * 1e6. Per week = 1e6 ($1).
    // Recovers $1 per week -> 1 GLW.
    let r_long = w1
        .farm_rewards
        .iter()
        .find(|r| r.farm_id == "Long")
        .unwrap();
    let glw = r_long.assets.iter().find(|a| a.asset_id == "GLW").unwrap();
    assert_eq!(glw.asset_earned, scale_1e18());

    // Check last week for Long farm (week 4096)
    let w_last = output.get("4096").expect("last week exists");
    let r_long_last = w_last
        .farm_rewards
        .iter()
        .find(|r| r.farm_id == "Long")
        .unwrap();
    assert_eq!(r_long_last.assets[0].asset_earned, scale_1e18());
}

#[test]
fn test_scenario_7_glw_inflation_distribution() {
    // Multi-asset farms should receive GLW inflation proportional to their total deposit contribution
    // Verify GLW inflation is NOT duplicated across sub-farms

    // F1: $100 in GLW.
    // F2: $100 in GLW + $100 in USDG.
    // Region 1 Inflation: 120,641 GLW.
    // GLW Comp: F1($100) vs F2($100). Total $200. F2 gets 50% of GLW-comp inflation.
    // USDG Comp: F2($100). Total $100. F2 gets 100% of USDG-comp inflation.

    // How is region inflation split between comps?
    // Proportional to total deposits.
    // Total Region Deposits = $200 (GLW comp) + $100 (USDG comp) = $300.
    // GLW Comp gets 2/3 of inflation.
    // USDG Comp gets 1/3 of inflation.

    // F1 Reward: 50% of (2/3) = 1/3 of total inflation.
    // F2 Reward: 50% of (2/3) + 100% of (1/3) = 1/3 + 1/3 = 2/3 of total inflation.

    let f1 = MultiAssetSolarFarm {
        farm_id: "F1".to_string(),
        region_id: 1,
        net_weekly_impact_assets: scale_1e18(),
        total_protocol_deposit_value: BigInt::from(100) * scale_1e6(),
        first_week: 1,
        weeks_alive: 5,
        assets: vec![AssetRequirement {
            asset_id: "GLW".to_string(),
            assets_required: BigInt::from(100) * scale_1e18(),
            assets_required_usdc: BigInt::from(100) * scale_1e6(),
            quoted_by_gve_price_per_asset: scale_1e6(),
            decimals: Some(18),
        }],
        reward_split: make_split(),
    };

    let f2 = MultiAssetSolarFarm {
        farm_id: "F2".to_string(),
        region_id: 1,
        net_weekly_impact_assets: scale_1e18(),
        total_protocol_deposit_value: BigInt::from(200) * scale_1e6(),
        first_week: 1,
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
                assets_required: BigInt::from(100) * scale_1e6(),
                assets_required_usdc: BigInt::from(100) * scale_1e6(),
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(6),
            },
        ],
        reward_split: make_split(),
    };

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f1, f2],
        gctl_distribution: None,
        output_farms: None,
    };

    let (output, _) = simulate_multi_asset(input, false).unwrap();
    let w1 = output.get("1").unwrap();

    let r1 = w1.farm_rewards.iter().find(|r| r.farm_id == "F1").unwrap();
    let r2 = w1.farm_rewards.iter().find(|r| r.farm_id == "F2").unwrap();

    let inf1 = &r1.glow_inflation_reward;
    let inf2 = &r2.glow_inflation_reward;

    let total = inf1 + inf2;
    let expected_total = BigInt::from(120_641) * scale_1e18();

    // Check total
    assert!((&total - &expected_total).abs() < scale_1e18());

    // Check Ratio F2 should be 2x F1
    // inf2 ~ 2 * inf1
    let ratio = (inf2 * BigInt::from(100)) / inf1;
    // 200 +/- small error
    assert!(
        ratio > BigInt::from(199) && ratio < BigInt::from(201),
        "F2 should get 2x inflation of F1"
    );
}
