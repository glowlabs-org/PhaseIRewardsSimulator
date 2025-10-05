use crate::competition_simulator::simulate_with_diagnostics;
use crate::models::{InputData, RewardSplit, SolarFarm};
use num_bigint::BigInt;
use num_traits::FromPrimitive;

fn sf(
    id: &str,
    region: u64,
    asset: &str,
    ia: u64,
    pd: u64,
    first_week: u64,
    weeks: u64,
) -> SolarFarm {
    SolarFarm {
        farm_id: id.into(),
        asset_id: asset.into(),
        region_id: region,
        weekly_impact_assets: BigInt::from_u64(ia).unwrap(),
        protocol_deposit_value: BigInt::from_u64(pd).unwrap(),
        assets_required: BigInt::from_u64(pd).unwrap(), // any positive value
        reward_split: vec![RewardSplit {
            wallet_address: "0x0000000000000000000000000000000000000001".into(),
            glow_split_percent_6_decimals: BigInt::from(1_000_000),
            deposit_split_percent_6_decimals: BigInt::from(1_000_000),
        }],
        first_week,
        weeks_alive: weeks,
    }
}

fn s18() -> BigInt {
    BigInt::from(1_000_000_000_000_000_000u128)
}

#[test]
fn gctl_distributes_within_region_proportionally() {
    // Two competitions in CGP (glw + usdg) during week 1 with deposits 100 and 300.
    // Weekly CGP GLW is 120,641 scaled by 1e18. Shares are proportional to deposits:
    //   glw_share_glw = 120641e18 * 100 / 400 = 30160.25e18 (integer BigInt)
    //   glw_share_usdg = 120641e18 * 300 / 400 = 90480.75e18 (integer BigInt)
    let farms = vec![
        sf("A", 1, "glw", 1, 200, 1, 2),  // deposit per week = 100
        sf("B", 1, "usdg", 1, 600, 1, 2), // deposit per week = 300
    ];
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };

    let diag = simulate_with_diagnostics(input).expect("simulation ok");
    let cgp_glw = diag
        .competitions
        .iter()
        .find(|c| c.region_id == 1 && c.asset_id == "glw")
        .expect("cgp/glw competition present");
    let cgp_usdg = diag
        .competitions
        .iter()
        .find(|c| c.region_id == 1 && c.asset_id == "usdg")
        .expect("cgp/usdg competition present");

    let b1_glw = cgp_glw
        .buckets
        .iter()
        .find(|b| b.week_number == 1)
        .expect("week 1 present");
    let b1_usdg = cgp_usdg
        .buckets
        .iter()
        .find(|b| b.week_number == 1)
        .expect("week 1 present");

    assert_eq!(b1_glw.total_deposits, BigInt::from(100u32));
    assert_eq!(b1_usdg.total_deposits, BigInt::from(300u32));

    let weekly_total = BigInt::from(120_641u64) * s18();
    let expected_glw = (&weekly_total * BigInt::from(100u32)) / BigInt::from(400u32);
    let expected_usdg = (&weekly_total * BigInt::from(300u32)) / BigInt::from(400u32);

    assert_eq!(b1_glw.glw_inflation, expected_glw);
    assert_eq!(b1_usdg.glw_inflation, expected_usdg);
}

#[test]
fn gctl_single_competition_gets_full_allocation() {
    // Only one competition in Utah for week 5 => it should receive full 18,119 GLW (scaled 1e18).
    let farms = vec![sf("U1", 2, "usdg", 1, 1000, 5, 2)];
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };

    let diag = simulate_with_diagnostics(input).expect("simulation ok");
    let comp = diag
        .competitions
        .iter()
        .find(|c| c.region_id == 2 && c.asset_id == "usdg")
        .expect("utah/usdg present");
    let b5 = comp
        .buckets
        .iter()
        .find(|b| b.week_number == 5)
        .expect("week 5 present");
    assert_eq!(b5.glw_inflation, BigInt::from(18_119u32) * s18());
}

#[test]
fn gctl_zero_total_deposits_skips_distribution() {
    // Missouri region, two competitions but both with zero deposit contribution in week 10.
    // PD=1 over 10 weeks => per-bucket deposit = 0.
    let farms = vec![
        sf("M1", 4, "glw", 1, 1, 10, 10),
        sf("M2", 4, "usdg", 1, 1, 10, 10),
    ];
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };
    let diag = simulate_with_diagnostics(input).expect("simulation ok");
    let mo_glw = diag
        .competitions
        .iter()
        .find(|c| c.region_id == 4 && c.asset_id == "glw")
        .expect("missouri/glw present");
    let mo_usdg = diag
        .competitions
        .iter()
        .find(|c| c.region_id == 4 && c.asset_id == "usdg")
        .expect("missouri/usdg present");

    let b10_glw = mo_glw
        .buckets
        .iter()
        .find(|b| b.week_number == 10)
        .expect("week 10 present");
    let b10_usdg = mo_usdg
        .buckets
        .iter()
        .find(|b| b.week_number == 10)
        .expect("week 10 present");

    assert_eq!(b10_glw.total_deposits, BigInt::from(0u8));
    assert_eq!(b10_usdg.total_deposits, BigInt::from(0u8));
    assert_eq!(b10_glw.glw_inflation, BigInt::from(0u8));
    assert_eq!(b10_usdg.glw_inflation, BigInt::from(0u8));
}

#[test]
fn gctl_unconfigured_region_gets_no_inflation() {
    let farms = vec![sf("R1", 99999, "glw", 2, 200, 3, 2)];
    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };
    let diag = simulate_with_diagnostics(input).expect("simulation ok");
    let comp = diag
        .competitions
        .iter()
        .find(|c| c.region_id == 99999)
        .expect("unknown region comp present");
    let b3 = comp
        .buckets
        .iter()
        .find(|b| b.week_number == 3)
        .expect("week 3 present");
    assert_eq!(b3.glw_inflation, BigInt::from(0u8));
}
