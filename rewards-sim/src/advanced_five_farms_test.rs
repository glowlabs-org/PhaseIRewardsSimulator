use crate::competition_simulator::simulate;
use crate::models::{InputData, RewardSplit, SolarFarm};
use crate::test_utils::{assert_both_endpoints_status, write_log};
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use serde_json::json;

#[test]
fn advanced_five_farms_single_region() {
    let scale = BigInt::from_u64(1_000_000_000_000_000_000).unwrap();

    let proto = BigInt::from_u64(1000).unwrap() * &scale;
    let assets = BigInt::from_u64(1000).unwrap() * &scale;

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
            region_id: 2,
            weekly_impact_assets: BigInt::from_u64(5).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets.clone(),
            reward_split: vec![RewardSplit {
                wallet_address: addrs[0].into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
            first_week: 2,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F2".into(),
            asset_id: "usdg".into(),
            region_id: 2,
            weekly_impact_assets: BigInt::from_u64(6).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets.clone(),
            reward_split: vec![RewardSplit {
                wallet_address: addrs[1].into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
            first_week: 3,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F3".into(),
            asset_id: "usdg".into(),
            region_id: 2,
            weekly_impact_assets: BigInt::from_u64(7).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets.clone(),
            reward_split: vec![RewardSplit {
                wallet_address: addrs[2].into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
            first_week: 4,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F4".into(),
            asset_id: "usdg".into(),
            region_id: 2,
            weekly_impact_assets: BigInt::from_u64(8).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets.clone(),
            reward_split: vec![RewardSplit {
                wallet_address: addrs[3].into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
            first_week: 4,
            weeks_alive: 5,
        },
        SolarFarm {
            farm_id: "F5".into(),
            asset_id: "usdg".into(),
            region_id: 2,
            weekly_impact_assets: BigInt::from_u64(9).unwrap() * &scale,
            protocol_deposit_value: proto.clone(),
            assets_required: assets.clone(),
            reward_split: vec![RewardSplit {
                wallet_address: addrs[4].into(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000),
            }],
            first_week: 6,
            weeks_alive: 5,
        },
    ];

    let input = InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    };

    let input_for_log = input.clone();
    let sim_res = simulate(input.clone());
    let out = match sim_res {
        Ok(o) => o,
        Err(e) => {
            let err_json = json!({ "error": e.to_string() });
            write_log(
                "advanced_five_farms_single_region_error",
                &input_for_log,
                &err_json,
            );
            panic!("simulation should succeed, got error: {}", e);
        }
    };

    assert_eq!(out.total_regions, 1, "single-region competition expected");
    assert_eq!(
        out.weekly_rewards.len(),
        9,
        "weeks 2..=10 should be present"
    );

    let expected_counts = [
        (2_u64, 1_usize),
        (3, 2),
        (4, 4),
        (5, 4),
        (6, 5),
        (7, 4),
        (8, 3),
        (9, 1),
        (10, 1),
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
    write_log(
        "advanced_five_farms_single_region",
        &input_for_log,
        &out_json,
    );
}
