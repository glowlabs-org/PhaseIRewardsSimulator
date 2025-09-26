use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Deserialize)]
pub struct InputData {
    pub cgp_leftovers: HashMap<u64, BigInt>,
    pub solar_farms: Vec<SolarFarm>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SolarFarm {
    pub farm_id: String,
    pub asset_id: String,
    pub region_id: String,
    pub weekly_carbon_credits: BigInt,
    pub protocol_deposit_value: BigInt,
    pub assets_required: BigInt,
    pub rewards_address: String,
    pub first_week: u64,
    pub weeks_alive: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct OutputData {
    pub total_regions: usize,
    pub regional_stats: Vec<RegionStats>,
    pub weekly_rewards: Vec<WeekRewards>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct RegionStats {
    pub region: String,
    pub assets: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WeekRewards {
    pub week_number: u64,
    pub per_farm_rewards: Vec<FarmReward>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct FarmReward {
    pub farm_id: String,
    pub asset_id: String,
    pub region_id: String,
    pub amount: BigInt,
    pub rewards_address: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompetitionID {
    pub region_id: String,
    pub asset_id: String,
}
impl Hash for CompetitionID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.region_id.hash(state);
        self.asset_id.hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct Competition {
    pub first_week: u64,
    pub final_week: u64,
    pub buckets: BTreeMap<u64, Bucket>,
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
    if s.len() != 42 || !s.starts_with("0x") {
        return false;
    }
    s.as_bytes()[2..].iter().all(u8::is_ascii_hexdigit)
}

pub fn unique_regions_and_assets(
    comps: &BTreeMap<CompetitionID, Competition>,
) -> (usize, Vec<RegionStats>) {
    let mut region_assets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for cid in comps.keys() {
        region_assets
            .entry(cid.region_id.clone())
            .or_default()
            .insert(cid.asset_id.clone());
    }
    let total_regions = region_assets.len();
    let mut stats = Vec::with_capacity(total_regions);
    for (region, assets) in region_assets {
        stats.push(RegionStats {
            region,
            assets: assets.into_iter().collect(),
        });
    }
    (total_regions, stats)
}
