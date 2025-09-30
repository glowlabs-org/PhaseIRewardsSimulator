use alloy_primitives::Address;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputData {
    #[serde(default, deserialize_with = "crate::serde_utils::de_leftovers_map")]
    pub cgp_leftovers: HashMap<u64, BigInt>,
    pub solar_farms: Vec<SolarFarm>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarFarm {
    pub farm_id: String,
    pub asset_id: String,
    pub region_id: String,
    #[serde(deserialize_with = "crate::serde_utils::de_bigint")]
    pub weekly_carbon_credits: BigInt,
    #[serde(deserialize_with = "crate::serde_utils::de_bigint")]
    pub protocol_deposit_value: BigInt,
    #[serde(deserialize_with = "crate::serde_utils::de_bigint")]
    pub assets_required: BigInt,
    pub rewards_address: String,
    pub first_week: u64,
    pub weeks_alive: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutputData {
    pub total_regions: usize,
    pub regional_stats: Vec<RegionStats>,
    pub weekly_rewards: Vec<WeekRewards>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegionStats {
    pub region: String,
    pub assets: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WeekRewards {
    pub week_number: u64,
    pub per_farm_rewards: Vec<FarmReward>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FarmReward {
    pub farm_id: String,
    pub asset_id: String,
    pub region_id: String,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub amount: BigInt,
    pub rewards_address: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompetitionID {
    pub region_id: String,
    pub asset_id: String,
}

#[derive(Clone, Debug)]
pub struct Competition {
    pub first_week: u64,
    pub final_week: u64,
    pub buckets: HashMap<u64, Bucket>,
    pub farms: HashMap<String, FarmInfo>,
}

#[derive(Clone, Debug)]
pub struct Bucket {
    pub total_deposits: BigInt,
    pub total_carbon_credits: BigInt,
    pub first_week_farms: Vec<String>,
    pub ongoing_farms: Vec<String>,
    pub last_week_farms: Vec<String>,
    pub farm_states: HashMap<String, FarmBucketState>,
    pub pool_net_assets: BigInt,
    pub pool_net_deposits: BigInt,
}

#[derive(Clone, Debug)]
pub struct FarmBucketState {
    pub deposits_contributed: BigInt,
    pub carbon_credits_contributed: BigInt,
    pub accumulated_drawdown: BigInt,
    pub net_overperformance: BigInt,
    pub rewards_this_week: BigInt,
}

#[derive(Clone, Debug)]
pub struct FarmInfo {
    pub farm_id: String,
    pub protocol_deposit_value: BigInt,
    pub assets_required: BigInt,
    pub first_week: u64,
    pub final_week: u64,
    pub rewards_address: String,
    pub asset_id: String,
    pub region_id: String,
}

pub fn is_valid_eth_address(s: &str) -> bool {
    let s = s.trim();
    if s.len() != 42 {
        return false;
    }
    if !(s.starts_with("0x") || s.starts_with("0X")) {
        return false;
    }
    s.parse::<Address>().is_ok()
}

pub fn unique_regions_and_assets(
    comps: &HashMap<CompetitionID, Competition>,
) -> (usize, Vec<RegionStats>) {
    let mut region_assets: HashMap<String, HashSet<String>> = HashMap::new();
    for cid in comps.keys() {
        region_assets
            .entry(cid.region_id.clone())
            .or_default()
            .insert(cid.asset_id.clone());
    }
    let total_regions = region_assets.len();
    let mut stats: Vec<RegionStats> = region_assets
        .into_iter()
        .map(|(region, assets)| {
            let mut assets_vec: Vec<String> = assets.into_iter().collect();
            assets_vec.sort();
            RegionStats {
                region,
                assets: assets_vec,
            }
        })
        .collect();
    stats.sort_by(|a, b| a.region.cmp(&b.region));
    (total_regions, stats)
}
