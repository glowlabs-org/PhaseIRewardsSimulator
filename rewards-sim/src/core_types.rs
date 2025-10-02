use crate::models::RewardSplit;
use num_bigint::BigInt;
use std::collections::HashMap;

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
    pub total_impact_assets: BigInt,
    pub first_week_farms: Vec<String>,
    pub ongoing_farms: Vec<String>,
    pub last_week_farms: Vec<String>,
    pub farm_states: HashMap<String, FarmBucketState>,
    pub pool_net_assets: BigInt,
    pub pool_net_deposits: BigInt,
    pub glw_inflation: BigInt,
}

#[derive(Clone, Debug)]
pub struct FarmBucketState {
    pub deposits_contributed: BigInt,
    pub impact_assets_contributed: BigInt,
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
    pub rewards_address: Option<String>,
    pub asset_id: String,
    pub region_id: String,
    pub reward_splits: Vec<RewardSplit>,
}
