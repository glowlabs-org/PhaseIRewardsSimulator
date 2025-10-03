use crate::competition_simulator::simulate_with_diagnostics;
use crate::models::{InputData, SolarFarm};
use crate::test_utils::assert_both_endpoints_status;
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::FromPrimitive;

#[test]
fn default_frontend_farms_have_no_consistency_issues() {
    // Weekly IA values (scaled to 1e18): 0.08, 0.10, 0.12
    let ia1 = BigInt::from_u128(80_000_000_000_000_000u128).unwrap();
    let ia2 = BigInt::from_u128(100_000_000_000_000_000u128).unwrap();
    let ia3 = BigInt::from_u128(120_000_000_000_000_000u128).unwrap();

    // Dollars scaled 1e6
    let pd1 = BigInt::from_u64(40_000).unwrap() * BigInt::from_u64(1_000_000).unwrap();
    let pd2 = BigInt::from_u64(80_000).unwrap() * BigInt::from_u64(1_000_000).unwrap();
    let pd3 = BigInt::from_u64(50_000).unwrap() * BigInt::from_u64(1_000_000).unwrap();

    // Prices scaled 1e6
    let p1 = BigInt::from_u64(300_000).unwrap(); // $0.30
    let p2 = BigInt::from_u64(400_000).unwrap(); // $0.40
    let p3 = BigInt::from_u64(400_000).unwrap(); // $0.40

    let scale18 = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();

    // assetsRequired = pd * 1e18 / price (for non-usdg asset)
    let ar1 = (&pd1 * &scale18) / &p1;
    let ar2 = (&pd2 * &scale18) / &p2;
    let ar3 = (&pd3 * &scale18) / &p3;

    let farms = vec![
        SolarFarm {
            farm_id: "1".into(),
            asset_id: "glw".into(),
            region_id: 123456,
            weekly_impact_assets: ia1,
            protocol_deposit_value: pd1,
            assets_required: ar1,
            rewards_address: None,
            reward_split: vec![],
            first_week: 1,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "2".into(),
            asset_id: "glw".into(),
            region_id: 123456,
            weekly_impact_assets: ia2,
            protocol_deposit_value: pd2,
            assets_required: ar2,
            rewards_address: None,
            reward_split: vec![],
            first_week: 2,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "3".into(),
            asset_id: "glw".into(),
            region_id: 123456,
            weekly_impact_assets: ia3,
            protocol_deposit_value: pd3,
            assets_required: ar3,
            rewards_address: None,
            reward_split: vec![],
            first_week: 2,
            weeks_alive: 5,
        },
    ];

    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };

    let diag = simulate_with_diagnostics(input.clone()).expect("simulation ok");
    assert!(
        diag.errors.is_empty(),
        "expected no diagnostics errors, got: {:?}",
        diag.errors
    );

    // Endpoints should accept this input with 200 OK
    assert_both_endpoints_status(&input, StatusCode::OK);
}

// Ensure USDG competition uses 1e6 scaling and deposits contributed are non-zero.
#[test]
fn usdg_deposits_contributed_nonzero_and_scaled() {
    let scale6 = BigInt::from_u64(1_000_000).unwrap();
    let scale18 = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();

    // Two farms in Utah/USDG, first week 96, 60 weeks, $6000 and $3000 deposits
    let pd_a = BigInt::from_u64(6_000).unwrap() * &scale6;
    let pd_b = BigInt::from_u64(3_000).unwrap() * &scale6;
    // USDG $1 peg => assetsRequired scaled 1e6 equals protocol deposit value
    let ar_a = pd_a.clone();
    let ar_b = pd_b.clone();

    let farms = vec![
        SolarFarm {
            farm_id: "A".into(),
            asset_id: "usdg".into(),
            region_id: 2,
            weekly_impact_assets: BigInt::from_u64(1).unwrap() * &scale18,
            protocol_deposit_value: pd_a.clone(),
            assets_required: ar_a,
            rewards_address: None,
            reward_split: vec![],
            first_week: 96,
            weeks_alive: 60,
        },
        SolarFarm {
            farm_id: "B".into(),
            asset_id: "usdg".into(),
            region_id: 2,
            weekly_impact_assets: BigInt::from_u64(1).unwrap() * &scale18,
            protocol_deposit_value: pd_b.clone(),
            assets_required: ar_b,
            rewards_address: None,
            reward_split: vec![],
            first_week: 96,
            weeks_alive: 60,
        },
    ];

    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };

    let diag = simulate_with_diagnostics(input.clone()).expect("simulation ok");
    assert!(
        diag.errors.is_empty(),
        "expected no diagnostics errors, got: {:?}",
        diag.errors
    );

    // Find utah/usdg competition and week 96 bucket, check deposits contributed = PD / weeks_alive
    let comp = diag
        .competitions
        .iter()
        .find(|c| c.region_id == 2 && c.asset_id == "usdg")
        .expect("utah/usdg competition present");
    let b96 = comp
        .buckets
        .iter()
        .find(|b| b.week_number == 96)
        .expect("week 96 present");
    // deposits per week expected
    let expected_a = &pd_a / BigInt::from_u64(60).unwrap();
    let expected_b = &pd_b / BigInt::from_u64(60).unwrap();
    let mut saw_a = false;
    let mut saw_b = false;
    for st in &b96.farm_states {
        if st.farm_id == "A" {
            assert!(st.deposits_contributed > BigInt::from(0u8));
            assert_eq!(st.deposits_contributed, expected_a);
            saw_a = true;
        }
        if st.farm_id == "B" {
            assert!(st.deposits_contributed > BigInt::from(0u8));
            assert_eq!(st.deposits_contributed, expected_b);
            saw_b = true;
        }
    }
    assert!(saw_a && saw_b, "missing expected farm states");

    // Endpoints should accept this input with 200 OK
    assert_both_endpoints_status(&input, StatusCode::OK);
}
