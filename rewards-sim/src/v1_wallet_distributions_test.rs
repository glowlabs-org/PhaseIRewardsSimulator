use crate::competition_simulator::{build_public_output_from_detailed, simulate_with_diagnostics};
use crate::models::{InputData, RewardSplit, SolarFarm};
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
        _ => "0x3333333333333333333333333333333333333333",
    }
}

#[test]
fn v1_like_data_wallet_distributions_present_and_correct_each_week() {
    // Two farms over 2 weeks, matching a "v1-like" structure:
    // - CGP/USDG competition (region 1) with 2-way reward splits (60/40)
    // - Utah/GLW competition (region 2) with 100% to addr1
    //
    // Choose parameters so that weekly asset rewards are exact integers:
    //  protocolDepositValue = 100 (scaled 1e6), weeksAlive=2 => per-week deposit = 50
    //  assetsRequired = 100 (scaled to 1e6 for usdg, 1e18 for glw)
    //  weeklyImpactAssets = 1e18 (any positive non-zero)
    //
    // Asset rewards per week from own vault equal to (deposit_recovered * assets_required / protocol_deposit_value)
    // which yields 50 each week (scaled appropriately).
    //
    // GLW inflation per week:
    //  - cgp region total = 120,641e18, only one competition => full allocation to cgp/usdg comp.
    //  - utah region total = 18,119e18, one competition => full allocation to utah/glw comp.
    //
    // Reward splits:
    //  Farm A (cgp/usdg): addr0=60%, addr1=40% for both deposit and glow splits
    //  Farm B (utah/glw): addr1=100% for both deposit and glow splits
    //
    // Expectations per week:
    //  addr0:
    //    assetsEarned.usdg = 60% of 50e6 = 30e6
    //    glowInflationEarned += 60% of 120641e18
    //  addr1:
    //    assetsEarned.usdg = 40% of 50e6 = 20e6
    //    assetsEarned.glw  = 100% of 50e18
    //    glowInflationEarned += 40% of 120641e18 + 100% of 18119e18

    let pd = BigInt::from_u64(100).unwrap() * s6();
    let weeks_alive = 2u64;
    let dep_per = &pd / BigInt::from_u64(weeks_alive).unwrap();
    assert_eq!(dep_per, BigInt::from_u64(50).unwrap() * s6());

    // Farm A: cgp/usdg (region 1)
    let farm_a = SolarFarm {
        farm_id: "45-bb".into(),
        asset_id: "usdg".into(),
        region_id: 1,
        weekly_impact_assets: BigInt::from_u64(1).unwrap() * s18(),
        protocol_deposit_value: pd.clone(),
        assets_required: BigInt::from_u64(100).unwrap() * s6(), // usdg scaled 1e6
        reward_split: vec![
            RewardSplit {
                wallet_address: addr(0).into(),
                glow_split_percent_6_decimals: BigInt::from_u64(600_000).unwrap(),
                deposit_split_percent_6_decimals: BigInt::from_u64(600_000).unwrap(),
            },
            RewardSplit {
                wallet_address: addr(1).into(),
                glow_split_percent_6_decimals: BigInt::from_u64(400_000).unwrap(),
                deposit_split_percent_6_decimals: BigInt::from_u64(400_000).unwrap(),
            },
        ],
        first_week: 96,
        weeks_alive,
    };

    // Farm B: utah/glw (region 2)
    let farm_b = SolarFarm {
        farm_id: "90-fa".into(),
        asset_id: "glw".into(),
        region_id: 2,
        weekly_impact_assets: BigInt::from_u64(1).unwrap() * s18(),
        protocol_deposit_value: pd.clone(),
        // For GLW asset, assets_required should be scaled 1e18
        assets_required: BigInt::from_u64(100).unwrap() * s18(),
        reward_split: vec![RewardSplit {
            wallet_address: addr(1).into(),
            glow_split_percent_6_decimals: BigInt::from_u64(1_000_000).unwrap(),
            deposit_split_percent_6_decimals: BigInt::from_u64(1_000_000).unwrap(),
        }],
        first_week: 96,
        weeks_alive,
    };

    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: vec![farm_a, farm_b],
        gctl_distribution: None,
        output_farms: None,
    };

    let diag = simulate_with_diagnostics(input).expect("simulation ok");
    let pub_out = build_public_output_from_detailed(&diag.competitions, &diag.errors);
    assert!(
        diag.errors.is_empty(),
        "no diagnostics warnings expected, got: {:?}",
        diag.errors
    );

    // Expected constants per week
    let usdg_week_total = BigInt::from_u64(50).unwrap() * s6();
    let glw_week_total_tokens = BigInt::from_u64(50).unwrap() * s18();
    let cgp_glw_week = BigInt::from_u64(120_641).unwrap() * s18();
    let utah_glw_week = BigInt::from_u64(18_119).unwrap() * s18();

    // Helper to get week map entry as (addr -> (assets_earned_map, glw_earned))
    fn extract_wallets(
        map: &BTreeMap<String, crate::competition_simulator::PublicWeekOutput>,
        week: u64,
    ) -> BTreeMap<String, (BTreeMap<String, BigInt>, BigInt)> {
        let mut out = BTreeMap::new();
        let wk = map.get(&week.to_string()).expect("week present");
        assert!(
            !wk.wallet_distributions.is_empty(),
            "walletDistributions must not be empty for week {week}"
        );
        for w in &wk.wallet_distributions {
            let mut ae: BTreeMap<String, BigInt> = BTreeMap::new();
            for (k, v) in &w.assets_earned {
                let bi = v.parse::<BigInt>().expect("parse bigint");
                ae.insert(k.clone(), bi);
            }
            out.insert(w.user_address.clone(), (ae, w.glow_inflation_earned.clone()));
        }
        out
    }

    for week in [96u64, 97u64] {
        let wallets = extract_wallets(&pub_out, week);
        // Should have exactly two unique wallets: addr0 and addr1
        assert_eq!(wallets.len(), 2, "expected 2 wallets for week {}", week);

        let (ae0, glw0) = wallets.get(addr(0)).expect("addr0 present");
        let (ae1, glw1) = wallets.get(addr(1)).expect("addr1 present");

        // addr0 expectations
        let exp_addr0_usdg = (&usdg_week_total * BigInt::from_u64(600_000).unwrap()) / s6();
        assert_eq!(
            ae0.get("usdg").cloned().unwrap_or_default(),
            exp_addr0_usdg,
            "addr0 usdg split"
        );
        assert!(
            !ae0.contains_key("glw"),
            "addr0 should not receive GLW asset from farm B"
        );
        let exp_addr0_glw_inf = (&cgp_glw_week * BigInt::from_u64(600_000).unwrap()) / s6();
        assert_eq!(*glw0, exp_addr0_glw_inf, "addr0 total GLW inflation");

        // addr1 expectations
        let exp_addr1_usdg = (&usdg_week_total * BigInt::from_u64(400_000).unwrap()) / s6();
        let exp_addr1_glw_tokens = glw_week_total_tokens.clone(); // 100% of 50e18
        assert_eq!(
            ae1.get("usdg").cloned().unwrap_or_default(),
            exp_addr1_usdg,
            "addr1 usdg split"
        );
        assert_eq!(
            ae1.get("glw").cloned().unwrap_or_default(),
            exp_addr1_glw_tokens,
            "addr1 glw asset split"
        );
        let exp_addr1_glw_inf =
            (&cgp_glw_week * BigInt::from_u64(400_000).unwrap()) / s6() + utah_glw_week.clone();
        assert_eq!(*glw1, exp_addr1_glw_inf, "addr1 total GLW inflation");
    }
}