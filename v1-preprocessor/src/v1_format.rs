use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct V1History {
    pub usdg_per_week: HashMap<String, String>,
    pub solar_farms: Vec<V1SolarFarm>,
    pub protocol_deposits: Vec<V1ProtocolDeposit>,
    pub migrating_to_utah: Vec<V1MigratingToUtah>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct V1SolarFarm {
    pub farm_id: String,
    pub first_rewards_week: u64,
    pub net_weekly_carbon_credits: f64,
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
