use crate::competition_simulator::simulate_with_diagnostics;
use crate::models::{InputData, SolarFarm};
use crate::test_utils::{assert_both_endpoints_status, write_log};
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde_json::json;

#[test]
fn advanced_five_farms_single_region_specified() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();

    let proto = BigInt::from_u64(1_000).unwrap() * &scale;
    let assets_small = BigInt::from_u64(10).unwrap() * &scale;
    let assets_large = BigInt::from_u64(1_000).unwrap() * &scale;

    let addrs = [
        "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D",
        "0xa273164a466dbF9F0173996078fb382acC73F9E3",
        "0x0000000000000000000000000000000000000001",
        "0x0000000000000000000000000000000000000002",
        "0x0000000000000000000000000000000000000003",
    ];

    let farms = vec![
        SolarFarm {
            farm_id: "F1".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_impact_assets: BigInt::from_u64(20).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_small.clone(),
            rewards_address: Some(addrs[0].into()),
            reward_split: vec![],
            first_week: 1,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F2".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_impact_assets: BigInt::from_u64(20).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_small.clone(),
            rewards_address: Some(addrs[1].into()),
            reward_split: vec![],
            first_week: 2,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F3".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_impact_assets: BigInt::from_u64(2).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_large.clone(),
            rewards_address: Some(addrs[2].into()),
            reward_split: vec![],
            first_week: 3,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F4".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_impact_assets: BigInt::from_u64(20).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_small.clone(),
            rewards_address: Some(addrs[3].into()),
            reward_split: vec![],
            first_week: 4,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F5".into(),
            asset_id: "usdg".into(),
            region_id: "utah".into(),
            weekly_impact_assets: BigInt::from_u64(20).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets_small.clone(),
            rewards_address: Some(addrs[4].into()),
            reward_split: vec![],
            first_week: 5,
            weeks_alive: 5,
        },
    ];

    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };

    let input_for_log = input.clone();
    let diag = simulate_with_diagnostics(input.clone()).expect("simulation with diagnostics ok");
    let out = diag.output;

    assert_eq!(out.total_regions, 1, "expected single region");
    assert_eq!(
        out.weekly_rewards.len(),
        9,
        "expected contiguous weeks 1 through 9"
    );

    let expected_counts = [
        (1_u64, 1_usize),
        (2, 2),
        (3, 3),
        (4, 4),
        (5, 5),
        (6, 4),
        (7, 3),
        (8, 2),
        (9, 1),
    ];
    for (week, expected_len) in expected_counts {
        let wk = out
            .weekly_rewards
            .iter()
            .find(|w| w.week_number == week)
            .unwrap_or_else(|| panic!("missing week {week}"));
        assert_eq!(
            wk.per_farm_rewards.len(),
            expected_len,
            "unexpected farm count for week {week}"
        );
    }

    assert_both_endpoints_status(&input, StatusCode::OK);

    let out_json = serde_json::to_value(&out).expect("output to json");
    let log = json!({
        "diagnostics": {
            "consistency_errors": diag.errors,
        },
        "output": out_json
    });
    write_log(
        "advanced_five_farms_single_region_specified",
        &input_for_log,
        &log,
    );
}
