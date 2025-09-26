use num_bigint::BigInt;
use num_traits::{FromPrimitive, One};
use rewards_simulator::models::{InputData, SolarFarm};
use rewards_simulator::simulator::simulate;
use std::collections::HashMap;

#[test]
fn cgp_leftovers_bonus_applied() {
    // Two farms, equal carbon, equal deposit; total deposit per week 200.
    // Each recovers 100. Leftover 200 -> bonus per farm = 100*200/200 = 100
    let mut leftovers = HashMap::new();
    leftovers.insert(50_u64, BigInt::from_u64(200).unwrap());

    let input = InputData {
        cgp_leftovers: leftovers,
        solar_farms: vec![
            SolarFarm {
                farm_id: "F1".into(),
                asset_id: "usdg".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(100).unwrap(),
                assets_required: BigInt::from_u64(100).unwrap(),
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 50,
                weeks_alive: 1,
            },
            SolarFarm {
                farm_id: "F2".into(),
                asset_id: "usdg".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(100).unwrap(),
                assets_required: BigInt::from_u64(100).unwrap(),
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 50,
                weeks_alive: 1,
            },
        ],
    };

    let out = simulate(input).expect("ok");
    let wk = out
        .weekly_rewards
        .iter()
        .find(|w| w.week_number == 50)
        .unwrap();
    // base rewards: 100* (assets/deposit)=100*1=100
    // bonus: 100
    for r in &wk.per_farm_rewards {
        assert_eq!(r.amount, BigInt::from_u64(200).unwrap());
    }
}

#[test]
fn duplicate_farm_id_rejected() {
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "dup".into(),
                asset_id: "usdg".into(),
                region_id: "x".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10).unwrap(),
                assets_required: BigInt::from_u64(10).unwrap(),
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 1,
                weeks_alive: 1,
            },
            SolarFarm {
                farm_id: "dup".into(),
                asset_id: "usdg".into(),
                region_id: "x".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(10).unwrap(),
                assets_required: BigInt::from_u64(10).unwrap(),
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 1,
                weeks_alive: 1,
            },
        ],
    };
    assert!(simulate(input).is_err());
}

#[test]
fn zero_carbon_credits_rejected() {
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![SolarFarm {
            farm_id: "z".into(),
            asset_id: "a".into(),
            region_id: "r".into(),
            weekly_carbon_credits: BigInt::from_u64(0).unwrap(),
            protocol_deposit_value: BigInt::from_u64(10).unwrap(),
            assets_required: BigInt::from_u64(10).unwrap(),
            rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
            first_week: 1,
            weeks_alive: 1,
        }],
    };
    assert!(simulate(input).is_err());
}

#[test]
fn happy_path_multiple_regions() {
    let input = InputData {
        cgp_leftovers: HashMap::new(),
        solar_farms: vec![
            SolarFarm {
                farm_id: "A".into(),
                asset_id: "glw".into(),
                region_id: "cgp".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(100).unwrap(),
                assets_required: BigInt::from_u64(300).unwrap(), // 3x
                rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                first_week: 2,
                weeks_alive: 1,
            },
            SolarFarm {
                farm_id: "B".into(),
                asset_id: "usdg".into(),
                region_id: "utah".into(),
                weekly_carbon_credits: BigInt::one(),
                protocol_deposit_value: BigInt::from_u64(100).unwrap(),
                assets_required: BigInt::from_u64(200).unwrap(), // 2x
                rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                first_week: 2,
                weeks_alive: 1,
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
    assert_eq!(r_a.amount, BigInt::from_u64(300).unwrap());
    let r_b = wk
        .per_farm_rewards
        .iter()
        .find(|r| r.farm_id == "B")
        .unwrap();
    assert_eq!(r_b.amount, BigInt::from_u64(200).unwrap());
}
