use crate::errors::SimError;
use crate::models::InputData;
use std::collections::HashSet;

const V1_DATA_JSON: &str = include_str!("../v1-data.json");

pub fn load_and_merge_v1_data(user_input: InputData) -> Result<InputData, SimError> {
    let v1_input: InputData = serde_json::from_str(V1_DATA_JSON)
        .map_err(|e| SimError::internal(format!("failed to parse embedded v1-data.json: {e}")))?;

    merge_v1_data(user_input, v1_input)
}

pub fn merge_v1_data(user_input: InputData, v1_input: InputData) -> Result<InputData, SimError> {
    let mut new_input = user_input;

    // Merge cgpLeftovers
    for (week, amount) in v1_input.cgp_leftovers {
        let entry = new_input.cgp_leftovers.entry(week).or_default();
        *entry += amount;
    }

    // Merge solarFarms
    let user_farm_ids: HashSet<String> = new_input
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
        new_input.solar_farms.push(v1_farm);
    }

    Ok(new_input)
}
