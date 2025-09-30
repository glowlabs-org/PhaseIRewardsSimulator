use std::collections::HashMap;
use v1_preprocessor::processing::process_v1_history;
use v1_preprocessor::v1_format::{
    V1History, V1MigratingToUtah, V1ProtocolDeposit, V1RewardSplit, V1SolarFarm,
};

#[test]
fn test_initial_negative_cgp_leftover_pruned() {
    let v1_history = V1History {
        usdg_per_week: HashMap::from([("97".to_string(), "-5".to_string())]),
        solar_farms: HashMap::new(),
        protocol_deposits: vec![],
        migrating_to_utah: vec![],
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2 = result.unwrap();
    // Negative dust should be pruned from final output.
    assert!(v2.cgp_leftovers.get(&97).is_none());
}

fn build_full_weeks_map() -> HashMap<String, String> {
    let mut usdg_per_week: HashMap<String, String> = HashMap::new();
    for w in 97u64..=287u64 {
        // Provide a baseline value; week 97 can be overwritten by tests if needed.
        usdg_per_week.insert(w.to_string(), "100000".to_string());
    }
    usdg_per_week
}

fn basic_farm() -> (HashMap<String, V1SolarFarm>, Vec<V1MigratingToUtah>) {
    let mut solar_farms: HashMap<String, V1SolarFarm> = HashMap::new();
    solar_farms.insert(
        "farm-1".to_string(),
        V1SolarFarm {
            first_reward_week: 10,
            net_weekly_impact_assets: 0.5,
            reward_splits: vec![V1RewardSplit {
                wallet_address: "0x0123456789012345678901234567890123456789".to_string(),
                glow_split_percent_6_decimals: "1000000".to_string(),
                deposit_split_percent_6_decimals: "1000000".to_string(),
            }],
        },
    );
    (solar_farms, vec![])
}

#[test]
fn test_negative_exactly_minus_ten_pruned() {
    // Start with 10 at week 97, apply a deposit with ceil/usdg of 20 => leftover becomes -10.
    // -10 is within dust allowance during processing, but must be pruned from final output.
    let mut usdg_per_week = build_full_weeks_map();
    usdg_per_week.insert("97".to_string(), "10".to_string());
    let (solar_farms, migrating_to_utah) = basic_farm();
    let v1_history = V1History {
        usdg_per_week,
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "3840".to_string(), // ceil(3840/192)=20
            week_provided: 80,                 // affects week 97..
        }],
        migrating_to_utah,
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2 = result.unwrap();
    assert!(v2.cgp_leftovers.get(&97).is_none());
}

#[test]
fn test_negative_below_minus_ten_errors() {
    // Start with 10 at week 97, apply a deposit causing deduction 21 => leftover becomes -11, should error.
    let mut usdg_per_week = build_full_weeks_map();
    usdg_per_week.insert("97".to_string(), "10".to_string());
    let (solar_farms, migrating_to_utah) = basic_farm();
    let v1_history = V1History {
        usdg_per_week,
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "4032".to_string(), // ceil(4032/192)=21
            week_provided: 80,
        }],
        migrating_to_utah,
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}
