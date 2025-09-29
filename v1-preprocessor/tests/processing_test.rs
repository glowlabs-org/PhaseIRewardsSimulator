use std::collections::HashMap;
use v1_preprocessor::processing::process_v1_history;
use v1_preprocessor::v1_format::{
    V1History, V1MigratingToUtah, V1ProtocolDeposit, V1RewardSplit, V1SolarFarm,
};

fn get_test_v1_history() -> V1History {
    let mut solar_farms: HashMap<String, V1SolarFarm> = HashMap::new();
    solar_farms.insert(
        "45-ab".to_string(),
        V1SolarFarm {
            first_reward_week: 34,
            net_weekly_carbon_credits: 0.12,
            reward_splits: vec![V1RewardSplit {
                wallet_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".to_string(),
                glow_split_percent_6_decimals: "1000000".to_string(),
                deposit_split_percent_6_decimals: "1000000".to_string(),
            }],
        },
    );

    V1History {
        usdg_per_week: HashMap::from([
            ("96".to_string(), "12345".to_string()),
            ("97".to_string(), "23456".to_string()),
        ]),
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "45-ab".to_string(),
            usdg_provided: "10000".to_string(),
            week_provided: 32,
        }],
        migrating_to_utah: vec![V1MigratingToUtah {
            farm_id: "45-ab".to_string(),
            updated_protocol_deposit_value: "16000".to_string(),
        }],
    }
}

#[test]
fn happy_path_test() {
    let v1_history = get_test_v1_history();
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let mut v2_config = result.unwrap();

    // Sort farms for consistent test results
    v2_config
        .solar_farms
        .sort_by(|a, b| a.farm_id.cmp(&b.farm_id));

    assert_eq!(v2_config.cgp_leftovers.get("96").unwrap(), "12292");
    assert_eq!(v2_config.cgp_leftovers.get("97").unwrap(), "23403");

    assert_eq!(v2_config.solar_farms.len(), 1);
    let farm = &v2_config.solar_farms[0];
    assert_eq!(farm.farm_id, "45-ab");
    assert_eq!(farm.asset_id, "usdg");
    assert_eq!(farm.region_id, "utah");
    assert_eq!(farm.net_weekly_carbon_credits, "120000000000000000");
    assert_eq!(farm.protocol_deposit_value, "16000");
    assert_eq!(farm.assets_required, "10000");
    assert_eq!(farm.first_week, 96);
    assert_eq!(farm.weeks_alive, 71);
    assert_eq!(farm.reward_splits.len(), 1);
}

#[test]
fn test_empty_input() {
    let v1_history = V1History {
        usdg_per_week: HashMap::new(),
        solar_farms: HashMap::new(),
        protocol_deposits: vec![],
        migrating_to_utah: vec![],
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2_config = result.unwrap();
    assert!(v2_config.cgp_leftovers.is_empty());
    assert!(v2_config.solar_farms.is_empty());
}

#[test]
fn test_invalid_glow_split_sum() {
    let mut v1_history = get_test_v1_history();
    let key = "45-ab".to_string();
    v1_history.solar_farms.get_mut(&key).unwrap().reward_splits[0].glow_split_percent_6_decimals =
        "999999".to_string();
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}

#[test]
fn test_invalid_deposit_split_sum() {
    let mut v1_history = get_test_v1_history();
    let key = "45-ab".to_string();
    v1_history.solar_farms.get_mut(&key).unwrap().reward_splits[0]
        .deposit_split_percent_6_decimals = "1".to_string();
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}

#[test]
fn test_migrate_nonexistent_farm() {
    let mut v1_history = get_test_v1_history();
    v1_history.migrating_to_utah[0].farm_id = "non-existent".to_string();
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}

#[test]
fn test_deposit_to_evicted_farm() {
    let mut v1_history = get_test_v1_history();
    v1_history.protocol_deposits.push(V1ProtocolDeposit {
        corresponding_farm: "evicted-farm".to_string(),
        usdg_provided: "5000".to_string(),
        week_provided: 50,
    });
    let result = process_v1_history(v1_history);
    assert!(result.is_ok()); // Should be skipped, not an error
}

#[test]
fn test_invalid_number_in_week() {
    let mut v1_history = get_test_v1_history();
    v1_history
        .usdg_per_week
        .insert("not-a-number".to_string(), "1000".to_string());
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}

#[test]
fn test_invalid_number_in_amount() {
    let mut v1_history = get_test_v1_history();
    v1_history
        .usdg_per_week
        .insert("100".to_string(), "not-a-number".to_string());
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}

#[test]
fn test_protocol_deposit_before_week_96() {
    let mut v1_history = get_test_v1_history();
    // This deposit's effect window is 48..240, so it will affect weeks >= 96
    v1_history.protocol_deposits[0].week_provided = 32;
    // This deposit's effect window is 80..272, so it will affect weeks >= 96
    v1_history.protocol_deposits.push(V1ProtocolDeposit {
        corresponding_farm: "45-ab".to_string(),
        usdg_provided: "19200".to_string(),
        week_provided: 64,
    });
    // This deposit's effect window is 95..287, so it will affect weeks >= 96
    v1_history.protocol_deposits.push(V1ProtocolDeposit {
        corresponding_farm: "45-ab".to_string(),
        usdg_provided: "192".to_string(),
        week_provided: 79,
    });
    // This deposit's effect window is 96..288, so it will affect week 96 onwards
    v1_history.protocol_deposits.push(V1ProtocolDeposit {
        corresponding_farm: "45-ab".to_string(),
        usdg_provided: "384".to_string(),
        week_provided: 80,
    });

    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2_config = result.unwrap();
    // Original: 12345
    // Dep 1 (10000): deduction 53.
    // Dep 2 (19200): deduction 100.
    // Dep 3 (192): deduction 1.
    // Dep 4 (384): deduction 2.
    // Total deduction for week 96: 53 + 100 + 1 + 2 = 156
    // Expected: 12345 - 156 = 12189
    assert_eq!(v2_config.cgp_leftovers.get("96").unwrap(), "12189");
}

#[test]
fn test_missing_cgp_leftover_error() {
    // Only provide week 96 in cgpLeftovers; the deposit window should include week 97,
    // which is missing and should trigger an error.
    let mut solar_farms: HashMap<String, V1SolarFarm> = HashMap::new();
    solar_farms.insert(
        "farm-1".to_string(),
        V1SolarFarm {
            first_reward_week: 10,
            net_weekly_carbon_credits: 0.5,
            reward_splits: vec![V1RewardSplit {
                wallet_address: "0x0123456789012345678901234567890123456789".to_string(),
                glow_split_percent_6_decimals: "1000000".to_string(),
                deposit_split_percent_6_decimals: "1000000".to_string(),
            }],
        },
    );
    let v1_history = V1History {
        usdg_per_week: HashMap::from([("96".to_string(), "1000".to_string())]),
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "192".to_string(), // deduction 1
            week_provided: 80, // affects weeks 96.., including week 97 which is missing
        }],
        migrating_to_utah: vec![],
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}

#[test]
fn test_negative_cgp_leftover_error() {
    // Provide small leftover for week 96 and a deposit that deducts more than available, causing negative.
    let mut solar_farms: HashMap<String, V1SolarFarm> = HashMap::new();
    solar_farms.insert(
        "farm-1".to_string(),
        V1SolarFarm {
            first_reward_week: 10,
            net_weekly_carbon_credits: 1.23,
            reward_splits: vec![V1RewardSplit {
                wallet_address: "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string(),
                glow_split_percent_6_decimals: "1000000".to_string(),
                deposit_split_percent_6_decimals: "1000000".to_string(),
            }],
        },
    );
    let v1_history = V1History {
        usdg_per_week: HashMap::from([("96".to_string(), "10".to_string())]),
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "2000".to_string(), // ceil(2000/192)=11
            week_provided: 80,                 // affects week 96
        }],
        migrating_to_utah: vec![],
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}
