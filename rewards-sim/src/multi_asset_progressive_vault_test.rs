use crate::competition_simulator::{
    simulate_multi_asset, simulate_with_diagnostics, SimulationDiagnostics,
};
use crate::models::{
    AssetRequirement, InputData, InputDataMultiAsset, MultiAssetSolarFarm, RewardSplit, SolarFarm,
};
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use std::collections::HashMap;

// --- Helpers ---

fn scale_18(v: u64) -> BigInt {
    BigInt::from(v) * BigInt::from(1_000_000_000_000_000_000u64)
}

fn scale_6(v: u64) -> BigInt {
    BigInt::from(v) * BigInt::from(1_000_000u64)
}

fn default_split() -> Vec<RewardSplit> {
    vec![RewardSplit {
        wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".to_string(),
        glow_split_percent_6_decimals: BigInt::from(1_000_000),
        deposit_split_percent_6_decimals: BigInt::from(1_000_000),
    }]
}

// Expands multi-asset input into single-asset simulation for white-box testing
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
    let sa_input = InputData {
        cgp_leftovers: input.cgp_leftovers,
        solar_farms: expanded_farms,
        gctl_distribution: input.gctl_distribution,
        output_farms: None,
    };
    simulate_with_diagnostics(sa_input).expect("simulation failed")
}

fn create_farm(
    id: &str,
    impact_18: u64,
    deposit_usd_6: u64,
    assets: Vec<(&str, u64)>, // (AssetID, USD Value)
) -> MultiAssetSolarFarm {
    let mut reqs = Vec::new();
    for (aid, usd) in assets {
        let decimals = if aid == "GLW" { 18 } else { 6 };
        let scale = if aid == "GLW" {
            scale_18(1)
        } else {
            scale_6(1)
        };
        // Assume price = $1 for simplicity in mapping USD to Units unless GLW ($0.1)
        let price = if aid == "GLW" {
            scale_6(1) / 10
        } else {
            scale_6(1)
        };

        let units = (scale_6(usd) * scale) / price.clone();

        reqs.push(AssetRequirement {
            asset_id: aid.to_string(),
            assets_required: units,
            assets_required_usdc: scale_6(usd),
            quoted_by_gve_price_per_asset: price,
            decimals: Some(decimals),
        });
    }

    MultiAssetSolarFarm {
        farm_id: id.to_string(),
        region_id: 1,
        net_weekly_impact_assets: scale_18(impact_18),
        total_protocol_deposit_value: scale_6(deposit_usd_6),
        first_week: 10,
        weeks_alive: 10,
        assets: reqs,
        reward_split: default_split(),
    }
}

// --- Scenarios ---

#[test]
fn test_scenario_1_asymmetric_performance() {
    // F1: 50% GLW, 50% USDG. Overperforms in GLW, Underperforms in USDG.
    let f1 = create_farm("F1", 100, 200, vec![("GLW", 100), ("USDG", 100)]);
    // Competitors to skew impact:
    // GLW: F1 needs to be 80% impact. F1 has 50 impact (50% of 100). 50 is 80% of 62.5.
    // So Comp1 needs 12.5 impact.
    let c1 = create_farm("C1", 125, 1000, vec![("GLW", 100)]); // High deposit, low impact
                                                               // C1 Impact = 125 * (100/1000) = 12.5. Correct.

    // USDG: F1 needs to be 20% impact. F1 has 50 impact. 50 is 20% of 250.
    // So Comp2 needs 200 impact.
    let c2 = create_farm("C2", 200, 100, vec![("USDG", 100)]); // Normal deposit, High impact

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f1, c1, c2],
        gctl_distribution: None,
        output_farms: None,
    };
    let diag = get_internals(input);
    let glw_comp = diag
        .competitions
        .iter()
        .find(|c| c.asset_id == "GLW")
        .unwrap();
    let usdg_comp = diag
        .competitions
        .iter()
        .find(|c| c.asset_id == "USDG")
        .unwrap();

    let f1_glw = &glw_comp.buckets[0]
        .farm_states
        .iter()
        .find(|f| f.farm_id == "F1|GLW")
        .unwrap();
    assert!(f1_glw.net_overperformance > BigInt::zero());

    let f1_usdg = &usdg_comp.buckets[0]
        .farm_states
        .iter()
        .find(|f| f.farm_id == "F1|USDG")
        .unwrap();
    assert_eq!(f1_usdg.net_overperformance, BigInt::zero());
    assert!(f1_usdg.accumulated_drawdown > BigInt::zero()); // Penalty
}

#[test]
fn test_scenario_2_pool_consumption() {
    // Farm A: Exhausts vault, takes from pool.
    // Farm B: Underperforms, feeds pool.
    let fa = create_farm("FA", 200, 100, vec![("GLW", 100)]); // High impact/deposit ratio
    let fb = create_farm("FB", 0, 100, vec![("GLW", 100)]); // Zero impact, feeds pool

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![fa, fb],
        gctl_distribution: None,
        output_farms: None,
    };
    let diag = get_internals(input);
    let comp = &diag.competitions[0];

    // By the end, FA should have drawn down fully and taken from pool
    let last_bucket = comp.buckets.last().unwrap();
    let fa_state = last_bucket
        .farm_states
        .iter()
        .find(|s| s.farm_id == "FA|GLW")
        .unwrap();

    let diff = (&fa_state.accumulated_drawdown - scale_6(100)).abs();
    assert!(diff < scale_6(1)); // Fully drawn down

    // Check if pool was used (rewards > deposits recovered from own vault)
    // Actually, simple check: pool should be empty at end
    assert!(last_bucket.pool_net_deposits.abs() < scale_6(1));
}

#[test]
fn test_scenario_3_penalty_contribution() {
    // Farm with 0 impact should pay penalty equal to contribution shortfall
    let f = create_farm("F", 0, 100, vec![("GLW", 100)]);
    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f],
        gctl_distribution: None,
        output_farms: None,
    };
    let diag = get_internals(input);
    let b = &diag.competitions[0].buckets[0];
    let st = &b.farm_states[0];

    // Deposit 100 over 10 weeks = 10/week.
    // Recovered = 0.
    // Penalty = 10.
    let expected_drawdown = scale_6(10);
    assert_eq!(st.accumulated_drawdown, expected_drawdown);
    assert_eq!(b.pool_net_deposits, expected_drawdown);
}

#[test]
fn test_scenario_4_aggregation_consistency() {
    // Verify that the aggregated output matches the sum of internals
    let f = create_farm("F", 100, 200, vec![("GLW", 100), ("USDG", 100)]);
    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f],
        gctl_distribution: None,
        output_farms: None,
    };

    let diag = get_internals(input.clone());
    let (output, _) = simulate_multi_asset(input, false).unwrap();

    let w10_out = output.get("10").unwrap();
    let f_out = &w10_out.farm_rewards[0];

    let glw_internal = diag
        .competitions
        .iter()
        .find(|c| c.asset_id == "GLW")
        .unwrap()
        .buckets[0]
        .farm_states[0]
        .rewards_this_week
        .clone();

    let usdg_internal = diag
        .competitions
        .iter()
        .find(|c| c.asset_id == "USDG")
        .unwrap()
        .buckets[0]
        .farm_states[0]
        .rewards_this_week
        .clone();

    let glw_out = f_out
        .assets
        .iter()
        .find(|a| a.asset_id == "GLW")
        .unwrap()
        .asset_earned
        .clone();
    let usdg_out = f_out
        .assets
        .iter()
        .find(|a| a.asset_id == "USDG")
        .unwrap()
        .asset_earned
        .clone();

    assert_eq!(glw_internal, glw_out);
    assert_eq!(usdg_internal, usdg_out);
}

#[test]
fn test_scenario_5_lifecycle_guarantees() {
    // At the end of lifecycle, accumulated_drawdown ~= protocol_deposit and overperformance ~= 0
    let f = create_farm("F", 100, 100, vec![("GLW", 100)]);
    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f],
        gctl_distribution: None,
        output_farms: None,
    };
    let diag = get_internals(input);
    let last = diag.competitions[0].buckets.last().unwrap();
    let st = &last.farm_states[0];

    let diff = (&st.accumulated_drawdown - scale_6(100)).abs();
    assert!(diff < scale_6(1));
    assert!(st.net_overperformance.abs() < scale_6(1));
}

#[test]
fn test_scenario_6_output_filtering_bug_check() {
    // Explicitly verify that filtering uses farm_id, not composite id.
    // If it used composite ID, result would be empty.
    let f1 = create_farm("F1", 100, 100, vec![("GLW", 100)]);
    let f2 = create_farm("F2", 100, 100, vec![("GLW", 100)]);

    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f1, f2],
        gctl_distribution: None,
        output_farms: Some(vec!["F1".to_string()]),
    };

    let (output, _) = simulate_multi_asset(input, false).unwrap();
    let w10 = output.get("10").unwrap();

    assert_eq!(w10.farm_rewards.len(), 1);
    assert_eq!(w10.farm_rewards[0].farm_id, "F1");
}

#[test]
fn test_scenario_7_decimal_precision() {
    // Verify math holds with 6 decimals vs 18 decimals
    // USDG (6 dec)
    let f = create_farm("F", 100, 100, vec![("USDG", 100)]);
    let input = InputDataMultiAsset {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![f],
        gctl_distribution: None,
        output_farms: None,
    };
    let diag = get_internals(input);
    let st = &diag.competitions[0].buckets[0].farm_states[0];

    // Deposit 100 / 10 wks = 10.
    // Recover 10.
    // Asset price $1. Rewards = 10 USDG.
    // 10 * 1e6
    assert_eq!(st.rewards_this_week, scale_6(10));
}
