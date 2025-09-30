use crate::errors::SimError;
use crate::models::InputData;
use std::collections::HashSet;
use std::fs;

pub fn load_and_merge_v1_data(user_input: InputData) -> Result<InputData, SimError> {
    let v1_data_str = fs::read_to_string("v1-data.json")
        .map_err(|e| SimError::internal(format!("failed to read v1-data.json: {e}")))?;
    let v1_input: InputData = serde_json::from_str(&v1_data_str)
        .map_err(|e| SimError::internal(format!("failed to parse v1-data.json: {e}")))?;

    merge_v1_data(user_input, v1_input)
}

pub fn merge_v1_data(
    mut user_input: InputData,
    v1_input: InputData,
) -> Result<InputData, SimError> {
    // Merge cgpLeftovers
    for (week, amount) in v1_input.cgp_leftovers {
        let entry = user_input.cgp_leftovers.entry(week).or_default();
        *entry += amount;
    }

    // Merge solarFarms
    let user_farm_ids: HashSet<String> = user_input
        .solar_farms
        .iter()
        .map(|f| f.farm_id.clone())
        .collect();
    for v1_farm in v1_input.solar_farms {
        if user_farm_ids.contains(&v1_farm.farm_id) {
            let fid = &v1_farm.farm_id;
            return Err(SimError::validation(format!(
                "duplicate farm id from v1 data: {fid}"
            )));
        }
        user_input.solar_farms.push(v1_farm);
    }

    Ok(user_input)
}
