use std::collections::HashMap;
use v1_preprocessor::processing::process_v1_history;
use v1_preprocessor::v1_format::{
    V1History, V1MigratingToUtah, V1ProtocolDeposit, V1RewardSplit, V1SolarFarm,
};

#[test]
fn test_initial_negative_cgp_leftover_pruned() {
    let v1_history = V1History {
        usdg_per_week: HashMap::from([
            ("97".to_string(), "1000".to_string()),
            ("98".to_string(), "-5".to_string()),
        ]),
        solar_farms: HashMap::new(),
        protocol_deposits: vec![],
        migrating_to_utah: vec![],
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2 = result.unwrap();
    assert!(v2.cgp_leftovers.is_empty());
}

fn build_full_weeks_map() -> HashMap<String, String> {
    let mut usdg_per_week: HashMap<String, String> = HashMap::new();
    for w in 97u64..=300u64 {
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
            reward_split: vec![V1RewardSplit {
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
    let mut usdg_per_week = build_full_weeks_map();
    usdg_per_week.insert("98".to_string(), "10".to_string());
    usdg_per_week.insert("99".to_string(), "10".to_string());
    usdg_per_week.insert("100".to_string(), "10".to_string());

    let (solar_farms, migrating_to_utah) = basic_farm();
    let v1_history = V1History {
        usdg_per_week,
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "2688".to_string(), // Was 3840. ceil(2688/192)=14. 10-14=-4. Merged becomes -8.
            week_provided: 82,                 // affects week 98..
        }],
        migrating_to_utah,
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2 = result.unwrap();
    assert!(v2.cgp_leftovers.get(&97).is_none());
}

#[test]
fn test_negative_cgp_leftover_in_deposit_calc_is_ok() {
    let mut usdg_per_week = build_full_weeks_map();
    usdg_per_week.insert("98".to_string(), "10".to_string());
    let (solar_farms, migrating_to_utah) = basic_farm();
    let v1_history = V1History {
        usdg_per_week,
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "4032".to_string(), // ceil(4032/192)=21. 10 - 21 = -11. This is > -20.
            week_provided: 82,                 // affects 98..
        }],
        migrating_to_utah,
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
}

#[test]
fn test_negative_below_minus_twenty_errors() {
    let mut usdg_per_week = build_full_weeks_map();
    usdg_per_week.insert("98".to_string(), "10".to_string());
    let (solar_farms, migrating_to_utah) = basic_farm();
    let v1_history = V1History {
        usdg_per_week,
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "5952".to_string(), // ceil(5952/192)=31. 10 - 31 = -21
            week_provided: 82,                 // affects 98..
        }],
        migrating_to_utah,
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}

#[test]
fn test_merged_negative_below_minus_twenty_errors() {
    let mut usdg_per_week = build_full_weeks_map();
    usdg_per_week.insert("98".to_string(), "10".to_string());
    usdg_per_week.insert("99".to_string(), "10".to_string());
    usdg_per_week.insert("100".to_string(), "10".to_string());
    let (solar_farms, migrating_to_utah) = basic_farm();
    let v1_history = V1History {
        usdg_per_week,
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "4032".to_string(), // ceil(4032/192)=21. 10 - 21 = -11.
            week_provided: 82,                 // affects 98..
        }],
        migrating_to_utah,
    };
    let result = process_v1_history(v1_history);
    // shifted[97]=-11, shifted[98]=-11, shifted[99]=-11
    // merged[97] = -11 + -11 + (-11*8/100) = -22.
    // This should error.
    assert!(result.is_err());
}
