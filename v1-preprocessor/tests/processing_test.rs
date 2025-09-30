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

    // Provide full range of weeks required by the test deposit:
    // For weekProvided=32, the window is [48, 240). We must provide >=96..=239.
    let mut usdg_per_week: HashMap<String, String> = HashMap::new();
    for w in 96u64..=239u64 {
        let v = if w == 96 {
            "12345".to_string()
        } else if w == 97 {
            "23456".to_string()
        } else {
            "100000".to_string()
        };
        usdg_per_week.insert(w.to_string(), v);
    }

    V1History {
        usdg_per_week,
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
    let v2_config = result.unwrap();

    assert_eq!(v2_config.cgp_leftovers.get(&96).unwrap(), "12292");
    assert_eq!(v2_config.cgp_leftovers.get(&97).unwrap(), "23403");

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

    // Extend cgpLeftovers to cover the longest window end (week 287)
    for w in 240u64..=287u64 {
        v1_history
            .usdg_per_week
            .insert(w.to_string(), "100000".to_string());
    }

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
    assert_eq!(v2_config.cgp_leftovers.get(&96).unwrap(), "12189");
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
fn test_negative_cgp_leftover_pruned() {
    // Provide full coverage to avoid missing-week errors; cause a small negative (-1) at week 96.
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

    let mut usdg_per_week: HashMap<String, String> = HashMap::new();
    for w in 96u64..=287u64 {
        let v = if w == 96 {
            "10".to_string()
        } else {
            "100000".to_string()
        };
        usdg_per_week.insert(w.to_string(), v);
    }

    let v1_history = V1History {
        usdg_per_week,
        solar_farms,
        protocol_deposits: vec![V1ProtocolDeposit {
            corresponding_farm: "farm-1".to_string(),
            usdg_provided: "2000".to_string(), // ceil(2000/192)=11
            week_provided: 80,                 // affects week 96
        }],
        migrating_to_utah: vec![],
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2 = result.unwrap();
    // Negative dust should be pruned from final output.
    assert!(v2.cgp_leftovers.get(&96).is_none());
}

#[test]
fn test_cgp_leftovers_sorted_numeric_and_filtered() {
    // Provide out-of-order numeric string keys to ensure numeric sort in output
    // and verify that keys < 96 are removed.
    let v1_history = V1History {
        usdg_per_week: HashMap::from([
            ("2".to_string(), "200".to_string()),
            ("10".to_string(), "1000".to_string()),
            ("1".to_string(), "100".to_string()),
            ("120".to_string(), "10000".to_string()),
            ("96".to_string(), "500".to_string()),
        ]),
        solar_farms: HashMap::new(),
        protocol_deposits: vec![],
        migrating_to_utah: vec![],
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2_config = result.unwrap();
    let keys: Vec<u64> = v2_config.cgp_leftovers.keys().copied().collect();
    assert_eq!(keys, vec![96, 120]);
}

#[test]
fn test_solar_farms_sorted_by_weeks_alive() {
    // Build three farms with differing first_reward_week to produce distinct weeksAlive
    let mut solar_farms: HashMap<String, V1SolarFarm> = HashMap::new();
    solar_farms.insert(
        "farm-early".to_string(),
        V1SolarFarm {
            first_reward_week: 10, // smallest weeksAlive
            net_weekly_carbon_credits: 0.1,
            reward_splits: vec![V1RewardSplit {
                wallet_address: "0x0000000000000000000000000000000000000001".to_string(),
                glow_split_percent_6_decimals: "1000000".to_string(),
                deposit_split_percent_6_decimals: "1000000".to_string(),
            }],
        },
    );
    solar_farms.insert(
        "farm-mid".to_string(),
        V1SolarFarm {
            first_reward_week: 34, // medium weeksAlive
            net_weekly_carbon_credits: 0.1,
            reward_splits: vec![V1RewardSplit {
                wallet_address: "0x0000000000000000000000000000000000000002".to_string(),
                glow_split_percent_6_decimals: "1000000".to_string(),
                deposit_split_percent_6_decimals: "1000000".to_string(),
            }],
        },
    );
    solar_farms.insert(
        "farm-late".to_string(),
        V1SolarFarm {
            first_reward_week: 100, // largest weeksAlive
            net_weekly_carbon_credits: 0.1,
            reward_splits: vec![V1RewardSplit {
                wallet_address: "0x0000000000000000000000000000000000000003".to_string(),
                glow_split_percent_6_decimals: "1000000".to_string(),
                deposit_split_percent_6_decimals: "1000000".to_string(),
            }],
        },
    );

    let v1_history = V1History {
        usdg_per_week: HashMap::from([
            ("96".to_string(), "100".to_string()),
            ("97".to_string(), "100".to_string()),
        ]),
        solar_farms,
        protocol_deposits: vec![],
        migrating_to_utah: vec![],
    };

    let result = process_v1_history(v1_history);
    assert!(result.is_ok());
    let v2_config = result.unwrap();

    // Ensure sorted by weeksAlive ascending, tie-broken by farmId
    let weeks: Vec<u64> = v2_config
        .solar_farms
        .iter()
        .map(|f| f.weeks_alive)
        .collect();
    let mut sorted_weeks = weeks.clone();
    sorted_weeks.sort();
    assert_eq!(weeks, sorted_weeks);

    // Also verify expected farm order by implied weeksAlive
    let ids: Vec<&str> = v2_config
        .solar_farms
        .iter()
        .map(|f| f.farm_id.as_str())
        .collect();
    assert_eq!(ids, vec!["farm-early", "farm-mid", "farm-late"]);
}
