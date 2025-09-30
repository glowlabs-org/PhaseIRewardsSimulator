use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct V2Configuration {
    pub cgp_leftovers: BTreeMap<u64, String>,
    pub solar_farms: Vec<V2SolarFarm>,
}

#[derive(Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct V2SolarFarm {
    pub farm_id: String,
    pub asset_id: String,
    pub region_id: String,
    pub net_weekly_impact_assets: String,
    pub protocol_deposit_value: String,
    pub assets_required: String,
    pub first_week: u64,
    pub weeks_alive: u64,
    pub reward_splits: Vec<V2RewardSplit>,
}

#[derive(Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct V2RewardSplit {
    pub wallet_address: String,
    pub glow_split_percent_6_decimals: String,
    pub deposit_split_percent_6_decimals: String,
}
