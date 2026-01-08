use crate::competition_simulator::{
    simulate_multi_asset, simulate_with_diagnostics, SimulationDiagnostics,
};
use crate::models::{
    AssetRequirement, InputData, InputDataMultiAsset, MultiAssetSolarFarm, RewardSplit, SolarFarm,
};
use num_bigint::BigInt;
use num_traits::Signed;
use std::collections::HashMap;

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

// Replicates the expansion logic of simulate_multi_asset to allow inspection of internal state
fn get_internals(input: InputDataMultiAsset) -> SimulationDiagnostics {
    let mut expanded_farms = Vec::new();
    for farm in input.solar_farms {
        for asset in &farm.assets {
            let virtual_id = format!("{}|{}", farm.farm_id, asset.asset_id);
            let impact = (&farm.net_weekly_impact_assets * &asset.assets_required_usdc)
                / &farm.total_protocol_deposit_value;

            expanded_farms.push(SolarFarm {
                farm_id: virtual_id,
                asset_id: asset.asset_id.clone(),
                region_id: farm.region_id,
                weekly_impact_assets: impact,
                protocol_deposit_value: asset.assets_required_usdc.clone(),
                assets_required: asset.assets_required.clone(),
                reward_split: farm.reward_split.clone(),
                first_week: farm.first_week,
                weeks_alive: farm.weeks_alive,
            });
        }
    }
    let single_asset_input = InputData {
        cgp_leftovers: input.cgp_leftovers,
        solar_farms: expanded_farms,
        gctl_distribution: input.gctl_distribution,
        output_farms: None,
    };
    simulate_with_diagnostics(single_asset_input).expect("simulation failed")
}

#[test]
fn test_asymmetric_performance() {
    // Scenario A: Farm 1 (GLW+USDG) overperforms in GLW, underperforms in USDG
    // F1: $100 GLW (50% of pool), $100 USDG (50% of pool). Total $200. Impact 1000.
    // F2 (GLW): $100. F3 (USDG): $100.
    // Target: F1 GLW impact 80% of total. F1 USDG impact 20% of total.
    // F1 GLW Impact = 500. Total GLW Impact = 625 -> F2 Impact = 125.
    // F1 USDG Impact = 500. Total USDG Impact = 2500 -> F3 Impact = 2000.

    let f1 = MultiAssetSolarFarm {
        farm_id: "F1".to_string(),
        region_id: 1,
        net_weekly_impact_assets: BigInt::from(1000) * scale_1e18(),
        total_protocol_deposit_value: BigInt::from(200) * scale_1e6(),
        first_week: 10,
        weeks_alive: 10,
        assets: vec![
            AssetRequirement {
                asset_id: "GLW".to_string(),
                assets_required: BigInt::from(1000) * scale_1e18(),
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

    let make_single = |id: &str, asset: &str, impact: BigInt| MultiAssetSolarFarm {
        farm_id: id.to_string(),
        region_id: 1,
        net_weekly_impact_assets: impact,
        total_protocol_deposit_value: BigInt::from(100) * scale_1e6(),
        first_week: 10,
        weeks_alive: 10,
        assets: vec![AssetRequirement {
            asset_id: asset.to_string(),
            assets_required: BigInt::from(100)
                * (if asset == "GLW" {
                    scale_1e18()
                } else {
                    scale_1e6()
                }),
            assets_required_usdc: BigInt::from(100) * scale_1e6(),
            quoted_by_gve_price_per_asset: scale_1e6(),
            decimals: Some(if asset == "GLW" { 18 } else { 6 }),
        }],
        reward_split: make_split(),
    };

    let f2 = make_single("F2", "GLW", BigInt::from(125) * scale_1e18());
    let f3 = make_single("F3", "USDG", BigInt::from(2000) * scale_1e18());

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f1, f2, f3],
        gctl_distribution: None,
        output_farms: None,
    };

    let diag = get_internals(input);
    let comp_glw = diag
        .competitions
        .iter()
        .find(|c| c.asset_id == "GLW")
        .unwrap();
    let comp_usdg = diag
        .competitions
        .iter()
        .find(|c| c.asset_id == "USDG")
        .unwrap();

    // Check Week 10 (First Week)
    let b_glw = &comp_glw.buckets[0];
    let st_f1_glw = b_glw
        .farm_states
        .iter()
        .find(|s| s.farm_id == "F1|GLW")
        .unwrap();
    // Overperformer: accumulates overperformance, does not touch pool yet
    assert!(st_f1_glw.net_overperformance > BigInt::from(0));
    // As it collects assets from its own vault (recovering principal), accumulated_drawdown increases.
    // This was previously checking for 0, which was incorrect as any recovery increases drawdown.
    assert!(st_f1_glw.accumulated_drawdown > BigInt::from(0));

    let b_usdg = &comp_usdg.buckets[0];
    let st_f1_usdg = b_usdg
        .farm_states
        .iter()
        .find(|s| s.farm_id == "F1|USDG")
        .unwrap();
    // Underperformer: no overperformance, accumulates drawdown (penalty)
    assert_eq!(st_f1_usdg.net_overperformance, BigInt::from(0));
    assert!(st_f1_usdg.accumulated_drawdown > BigInt::from(0));
}

#[test]
fn test_progressive_vault_invariants_and_pool_consistency() {
    // Tests Scenario 2 & 3: Invariants and Pool Consistency
    // Using a simpler setup with 2 farms in one competition to trace clearly
    let f1 = MultiAssetSolarFarm {
        farm_id: "A".to_string(),
        region_id: 1,
        net_weekly_impact_assets: BigInt::from(80) * scale_1e18(),
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
    // F2 has same deposit but less impact (underperformer)
    let f2 = MultiAssetSolarFarm {
        farm_id: "B".to_string(),
        region_id: 1,
        net_weekly_impact_assets: BigInt::from(20) * scale_1e18(),
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

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f1, f2],
        gctl_distribution: None,
        output_farms: None,
    };
    let diag = get_internals(input);
    let comp = &diag.competitions[0];

    // Check Invariants per bucket
    let deposit_per_week = BigInt::from(20) * scale_1e6(); // 100 / 5
    let total_deposit = BigInt::from(100) * scale_1e6();

    for b in &comp.buckets {
        // Pool Consistency
        // For GLW asset, tolerances are high (1e18), but for deposit it is 1e6
        let pool_net_dep = &b.pool_net_deposits;
        let pool_net_ast = &b.pool_net_assets;

        // Invariants for each farm
        for st in &b.farm_states {
            assert!(
                st.accumulated_drawdown <= total_deposit.clone() + BigInt::from(1000),
                "Drawdown exceeds deposit"
            );
            if st.farm_id == "A|GLW" {
                // A is overperformer
                assert!(st.net_overperformance >= BigInt::from(0));
            } else {
                // B is underperformer
                assert_eq!(st.net_overperformance, BigInt::from(0));
            }
        }
    }

    // Final week check
    let last_bucket = comp.buckets.last().unwrap();
    // After final week, pool should be approx zero
    assert!(last_bucket.pool_net_deposits.abs() < scale_1e6());
    assert!(last_bucket.pool_net_assets.abs() < scale_1e18());

    // Farms should be fully drawn down
    for st in &last_bucket.farm_states {
        let diff = (&st.accumulated_drawdown - &total_deposit).abs();
        assert!(diff < scale_1e6(), "Final drawdown not complete");
        assert!(
            st.net_overperformance.abs() < scale_1e6(),
            "Final overperformance not zero"
        );
    }
}

#[test]
fn test_impact_distribution_accuracy() {
    // Scenario 4: Uneven split (60/25/15)
    let f1 = MultiAssetSolarFarm {
        farm_id: "Split".to_string(),
        region_id: 1,
        net_weekly_impact_assets: BigInt::from(10000) * scale_1e18(),
        total_protocol_deposit_value: BigInt::from(100) * scale_1e6(),
        first_week: 10,
        weeks_alive: 5,
        assets: vec![
            AssetRequirement {
                asset_id: "GLW".to_string(),      // 60%
                assets_required: BigInt::from(60) * scale_1e18(), // $60 at $1/token
                assets_required_usdc: BigInt::from(60) * scale_1e6(),
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(18),
            },
            AssetRequirement {
                asset_id: "USDG".to_string(), // 25%
                assets_required: BigInt::from(25) * scale_1e6(), // $25 at $1/token
                assets_required_usdc: BigInt::from(25) * scale_1e6(),
                quoted_by_gve_price_per_asset: scale_1e6(),
                decimals: Some(6),
            },
            AssetRequirement {
                asset_id: "SGCTL".to_string(), // 15%
                assets_required: BigInt::from(15) * scale_1e6(), // $15 at $1/token
                assets_required_usdc: BigInt::from(15) * scale_1e6(),
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
    let diag = get_internals(input);

    let get_impact = |aid: &str| {
        let comp = diag
            .competitions
            .iter()
            .find(|c| c.asset_id == aid)
            .unwrap();
        let b = &comp.buckets[0];
        let st = &b.farm_states[0];
        st.impact_assets_contributed.clone()
    };

    let i_glw = get_impact("GLW");
    let i_usdg = get_impact("USDG");
    let i_sgctl = get_impact("SGCTL");

    assert_eq!(i_glw, BigInt::from(6000) * scale_1e18());
    assert_eq!(i_usdg, BigInt::from(2500) * scale_1e18());
    assert_eq!(i_sgctl, BigInt::from(1500) * scale_1e18());

    let total = i_glw + i_usdg + i_sgctl;
    assert_eq!(total, BigInt::from(10000) * scale_1e18());
}

#[test]
fn test_glw_inflation_distribution() {
    // Scenario 7: GLW Inflation
    // 2 farms in same competition. Inflation should split by deposits.
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
        total_protocol_deposit_value: BigInt::from(300) * scale_1e6(), // 3x deposit
        first_week: 1,
        weeks_alive: 5,
        assets: vec![AssetRequirement {
            asset_id: "GLW".to_string(),
            assets_required: BigInt::from(300) * scale_1e18(),
            assets_required_usdc: BigInt::from(300) * scale_1e6(),
            quoted_by_gve_price_per_asset: scale_1e6(),
            decimals: Some(18),
        }],
        reward_split: make_split(),
    };

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f1, f2],
        gctl_distribution: None,
        output_farms: None,
    };
    let (res, _) = simulate_multi_asset(input, false).unwrap();
    let w1 = res.get("1").unwrap();
    let r1 = w1.farm_rewards.iter().find(|r| r.farm_id == "F1").unwrap();
    let r2 = w1.farm_rewards.iter().find(|r| r.farm_id == "F2").unwrap();

    let inf1 = &r1.glow_inflation_reward;
    let inf2 = &r2.glow_inflation_reward;

    // F2 should have 3x inflation of F1 (approx)
    // Using integer math, 3 * inf1 should be close to inf2
    let ratio = (inf2 * BigInt::from(100)) / inf1;
    assert!(
        ratio >= BigInt::from(299) && ratio <= BigInt::from(301),
        "Inflation ratio mismatch"
    );

    // Total inflation for Region 1 is fixed (120,641 GLW)
    let total_inf = inf1 + inf2;
    // 120641 * 1e18
    let expected = BigInt::from(120_641) * scale_1e18();
    // allow dust
    let diff = (&total_inf - &expected).abs();
    assert!(diff < scale_1e18(), "Total inflation mismatch");
}
