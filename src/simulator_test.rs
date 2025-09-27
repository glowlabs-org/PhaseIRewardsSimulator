use crate::models::{InputData, SolarFarm};
use crate::simulator::simulate;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, One};
use std::collections::HashMap;

#[test]
fn cgp_leftovers_bonus_applied() {
    // Scale inputs to reduce dust impact
    let scale = BigInt::from_u64(1_000_000).unwrap();

    // Two farms, equal carbon, equal deposit; with weeks_alive=2:
    // per-week deposit per farm = 50, total per week = 100
    // Each recovers 50. Leftover 200 -> bonus per farm = 50*200/100 = 100
    // base own-vault rewards = 50 (ratio 1)
    // total per farm for week 50 = 150
    let mut leftovers = HashMap::new();
    leftovers.insert(50_u64, BigInt::from_u64(200).unwrap() * &scale);

    let input = InputData {
        cgp_leftovers: leftovers,
        solar_farms: vec![
            SolarFarm {
                farm_id: "F1".into(),
                asset_id: "usdg".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(100).unwrap() * &scale,
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 50,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "F2".into(),
                asset_id: "usdg".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(100).unwrap() * &scale,
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 50,
                weeks_alive: 2,
            },
        ],
    };

    let out = simulate(input).expect("ok");
    let wk = out
        .weekly_rewards
        .iter()
        .find(|w| w.week_number == 50)
        .unwrap();
    for r in &wk.per_farm_rewards {
        assert_eq!(r.amount, BigInt::from_u64(150).unwrap() * &scale);
    }
}

#[test]
fn duplicate_farm_id_rejected() {
    let scale = BigInt::from_u64(1_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "dup".into(),
                asset_id: "usdg".into(),
                region_id: "x".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 1,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "dup".into(),
                asset_id: "usdg".into(),
                region_id: "x".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
                assets_required: BigInt::from_u64(10).unwrap() * &scale,
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 1,
                weeks_alive: 2,
            },
        ],
    };
    assert!(simulate(input).is_err());
}

#[test]
fn zero_carbon_credits_rejected() {
    let scale = BigInt::from_u64(1_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "z".into(),
            asset_id: "a".into(),
            region_id: "r".into(),
            weekly_carbon_credits: BigInt::from_u64(0).unwrap(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
            assets_required: BigInt::from_u64(10).unwrap() * &scale,
            rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
            first_week: 1,
            weeks_alive: 2,
        }],
    };
    assert!(simulate(input).is_err());
}

#[test]
fn happy_path_multiple_regions() {
    let scale = BigInt::from_u64(1_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "A".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(300).unwrap() * &scale, // 3x
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 2,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "B".into(),
                asset_id: "usdg".into(),
                region_id: "utah".into(),
                weekly_carbon_credits: BigInt::one() * &scale,
                protocol_deposit_value: BigInt::from_u64(100).unwrap() * &scale,
                assets_required: BigInt::from_u64(200).unwrap() * &scale, // 2x
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 2,
                weeks_alive: 2,
            },
        ],
    };
    let out = simulate(input).expect("ok");
    assert_eq!(out.total_regions, 2);
    let wk = &out.weekly_rewards[0];
    assert_eq!(wk.week_number, 2);
    let r_a = wk
        .per_farm_rewards
        .iter()
        .find(|r| r.farm_id == "A")
        .unwrap();
    assert_eq!(r_a.amount, BigInt::from_u64(150).unwrap() * &scale);
    let r_b = wk
        .per_farm_rewards
        .iter()
        .find(|r| r.farm_id == "B")
        .unwrap();
    assert_eq!(r_b.amount, BigInt::from_u64(100).unwrap() * &scale);
}

#[test]
fn weeks_alive_minimum_enforced() {
    // weeks_alive = 1 should be rejected
    let scale = BigInt::from_u64(1_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "min1".into(),
            asset_id: "x".into(),
            region_id: "y".into(),
            weekly_carbon_credits: BigInt::one(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
            assets_required: BigInt::from_u64(10).unwrap() * &scale,
            rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
            first_week: 10,
            weeks_alive: 1,
        }],
    };
    assert!(simulate(input).is_err());
}

#[test]
fn weeks_alive_equal_two_allowed() {
    // Explicitly verify the new minimum weeks_alive=2 is accepted.
    let scale = BigInt::from_u64(1_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "ok2".into(),
            asset_id: "usdg".into(),
            region_id: "ok".into(),
            weekly_carbon_credits: BigInt::one(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap() * &scale,
            assets_required: BigInt::from_u64(10).unwrap() * &scale,
            rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
            first_week: 5,
            weeks_alive: 2,
        }],
    };
    assert!(simulate(input).is_ok());
}

#[test]
fn basic_build_and_simulate() {
    let scale = BigInt::from_u64(1_000_000).unwrap();
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "A".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10000).unwrap() * &scale,
                assets_required: BigInt::from_u64(20000).unwrap() * &scale, // 2 per unit
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 10,
                weeks_alive: 2,
            },
            SolarFarm {
                farm_id: "B".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10000).unwrap() * &scale,
                assets_required: BigInt::from_u64(20000).unwrap() * &scale,
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 10,
                weeks_alive: 2,
            },
        ],
    };
    let out = simulate(input).expect("ok");
    assert_eq!(out.total_regions, 1);
    assert_eq!(out.regional_stats.len(), 1);
    assert!(!out.weekly_rewards.is_empty());
    let wk10 = out
        .weekly_rewards
        .iter()
        .find(|w| w.week_number == 10)
        .unwrap();
    for r in &wk10.per_farm_rewards {
        assert_eq!(
            r.amount,
            BigInt::from_u64(10000).unwrap() * &BigInt::from_u64(1_000_000).unwrap()
        );
    }
}
