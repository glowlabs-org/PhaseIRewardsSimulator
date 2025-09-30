use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct V1History {
    pub usdg_per_week: HashMap<String, String>,
    // The v1 "solarFarms" is a map keyed by farmId, not an array.
    pub solar_farms: HashMap<String, V1SolarFarm>,
    pub protocol_deposits: Vec<V1ProtocolDeposit>,
    pub migrating_to_utah: Vec<V1MigratingToUtah>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct V1SolarFarm {
    // Matches "firstRewardWeek" (singular) in the input
    pub first_reward_week: u64,
    pub net_weekly_impact_assets: f64,
    pub reward_splits: Vec<V1RewardSplit>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct V1RewardSplit {
    pub wallet_address: String,
    pub glow_split_percent_6_decimals: String,
    pub deposit_split_percent_6_decimals: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct V1ProtocolDeposit {
    pub corresponding_farm: String,
    pub usdg_provided: String,
    pub week_provided: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct V1MigratingToUtah {
    pub farm_id: String,
    pub updated_protocol_deposit_value: String,
}
