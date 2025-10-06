use crate::competition_simulator::{build_public_output_from_detailed, simulate_with_diagnostics};
use crate::models::{InputData, RewardSplit, SolarFarm};
use crate::test_utils::assert_both_endpoints_status;
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use std::collections::BTreeMap;

fn s6() -> BigInt {
    BigInt::from(1_000_000u64)
}
fn s18() -> BigInt {
    BigInt::from(1_000_000_000_000_000_000u128)
}

fn addr(a: usize) -> &'static str {
    match a {
        0 => "0x1111111111111111111111111111111111111111",
        1 => "0x2222222222222222222222222222222222222222",
        2 => "0x3333333333333333333333333333333333333333",
        _ => "0x4444444444444444444444444444444444444444",
    }
}

#[test]
fn reward_splits_invariants_enforced() {
    // Bad: sums don't add to 1_000_000
    let farm = SolarFarm {
        farm_id: "bad".into(),
        asset_id: "usdg".into(),
        region_id: 2,
        weekly_impact_assets: BigInt::from_u64(1).unwrap() * s18(),
        protocol_deposit_value: BigInt::from_u64(100).unwrap() * s6(),
        assets_required: BigInt::from_u64(100).unwrap() * s6(),
        reward_split: vec![
            RewardSplit {
                wallet_address: addr(0).into(),
                glow_split_percent_6_decimals: BigInt::from(600_000u32),
                deposit_split_percent_6_decimals: BigInt::from(500_000u32),
            },
            RewardSplit {
                wallet_address: addr(1).into(),
                glow_split_percent_6_decimals: BigInt::from(399_999u32),
                deposit_split_percent_6_decimals: BigInt::from(500_000u32),
            },
        ],
        first_week: 1,
        weeks_alive: 2,
    };
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: vec![farm],
        gctl_distribution: None,
        output_farms: None,
    };
    assert_both_endpoints_status(&input, StatusCode::BAD_REQUEST);
}

#[test]
fn reward_splits_pass_through_and_not_synthesized() {
    // One farm, 2-way split. With single farm in region Utah/USDG, glw inflation is full share.
    // Asset rewards = AR / weeks = 100e6 / 2 = 50e6 each week.
    let farm = SolarFarm {
        farm_id: "F1".into(),
        asset_id: "usdg".into(),
        region_id: 2,
        weekly_impact_assets: BigInt::from_u64(1).unwrap() * s18(),
        protocol_deposit_value: BigInt::from_u64(100).unwrap() * s6(),
        assets_required: BigInt::from_u64(100).unwrap() * s6(),
        reward_split: vec![
            RewardSplit {
                wallet_address: addr(0).into(),
                glow_split_percent_6_decimals: BigInt::from(600_000u32),
                deposit_split_percent_6_decimals: BigInt::from(600_000u32),
            },
            RewardSplit {
                wallet_address: addr(1).into(),
                glow_split_percent_6_decimals: BigInt::from(400_000u32),
                deposit_split_percent_6_decimals: BigInt::from(400_000u32),
            },
        ],
        first_week: 1,
        weeks_alive: 2,
    };
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: vec![farm],
        gctl_distribution: None,
        output_farms: None,
    };
    let diag = simulate_with_diagnostics(input.clone()).expect("ok");
    // Build public output and check week "1"
    let pub_out = build_public_output_from_detailed(&diag.competitions, &diag.errors);
    assert!(diag.errors.is_empty(), "no diagnostics expected");
    let w1 = pub_out.get("1").expect("week 1 present");
    // Expect exactly two wallet distributions, one per reward split, and no synthesized rewardsAddress record.
    assert_eq!(w1.wallet_distributions.len(), 2);

    // Compute expected values
    let reward_total_week = BigInt::from_u64(50).unwrap() * s6(); // 50 usdg (scaled 1e6)
    let glw_week = BigInt::from_u64(18_119).unwrap() * s18(); // full utah allocation

    let mut map: BTreeMap<String, (BigInt, BigInt)> = BTreeMap::new();
    for wd in &w1.wallet_distributions {
        // assetsEarned must have only "usdg"
        assert!(wd.assets_earned.contains_key("usdg"));
        let val = wd
            .assets_earned
            .get("usdg")
            .unwrap()
            .parse::<BigInt>()
            .unwrap();
        let glw = wd.glow_inflation_earned.clone();
        map.insert(wd.user_address.clone(), (val, glw));
        // traces should list farm F1 only and reflect the corresponding splits
        for t in &wd.traces {
            assert_eq!(t.farm_id, "F1");
            assert_eq!(t.asset, "usdg");
        }
    }

    // Address 0 gets 60%
    let (a0_usdg, a0_glw) = map.get(addr(0)).expect("addr0 present");
    assert_eq!(
        *a0_usdg,
        (&reward_total_week * BigInt::from(600_000u32)) / s6()
    );
    assert_eq!(*a0_glw, (&glw_week * BigInt::from(600_000u32)) / s6());

    // Address 1 gets 40%
    let (a1_usdg, a1_glw) = map.get(addr(1)).expect("addr1 present");
    assert_eq!(
        *a1_usdg,
        (&reward_total_week * BigInt::from(400_000u32)) / s6()
    );
    assert_eq!(*a1_glw, (&glw_week * BigInt::from(400_000u32)) / s6());

    // Endpoints should accept this input
    assert_both_endpoints_status(&input, StatusCode::OK);
}
