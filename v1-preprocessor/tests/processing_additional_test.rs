use std::collections::HashMap;
use v1_preprocessor::processing::process_v1_history;
use v1_preprocessor::v1_format::V1History;

#[test]
fn test_initial_negative_cgp_leftover_error() {
    let v1_history = V1History {
        usdg_per_week: HashMap::from([("96".to_string(), "-5".to_string())]),
        solar_farms: HashMap::new(),
        protocol_deposits: vec![],
        migrating_to_utah: vec![],
    };
    let result = process_v1_history(v1_history);
    assert!(result.is_err());
}
