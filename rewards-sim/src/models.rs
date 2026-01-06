use crate::core_types::{Competition, CompetitionID};
use alloy_primitives::Address;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputData {
    #[serde(default, deserialize_with = "crate::serde_utils::de_leftovers_map")]
    pub cgp_leftovers: HashMap<u64, BigInt>,
    pub solar_farms: Vec<SolarFarm>,
    #[serde(default, deserialize_with = "crate::serde_utils::de_optional_gctl_map")]
    pub gctl_distribution: Option<HashMap<u64, BigInt>>,
    #[serde(default)]
    pub output_farms: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardSplit {
    pub wallet_address: String,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub glow_split_percent_6_decimals: BigInt,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub deposit_split_percent_6_decimals: BigInt,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarFarm {
    pub farm_id: String,
    pub asset_id: String,
    pub region_id: u64,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint",
        rename = "netWeeklyImpactAssets",
        alias = "weeklyImpactAssets",
        alias = "weeklyCarbonCredits"
    )]
    pub weekly_impact_assets: BigInt,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub protocol_deposit_value: BigInt,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub assets_required: BigInt,
    #[serde(default, rename = "rewardSplit")]
    pub reward_split: Vec<RewardSplit>,
    pub first_week: u64,
    pub weeks_alive: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutputData {
    pub total_regions: usize,
    pub regional_stats: Vec<RegionStats>,
    pub weekly_rewards: Vec<WeekRewards>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegionStats {
    pub region_id: u64,
    pub assets: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WeekRewards {
    pub week_number: u64,
    pub per_farm_rewards: Vec<FarmReward>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FarmReward {
    pub farm_id: String,
    pub asset_id: String,
    pub region_id: u64,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub amount: BigInt,
}

// --- Multi-Asset Types ---

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRequirement {
    pub asset_id: String,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub assets_required: BigInt,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint",
        rename = "assetsRequiredUSDC"
    )]
    pub assets_required_usdc: BigInt,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub quoted_by_gve_price_per_asset: BigInt,
    pub decimals: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiAssetSolarFarm {
    pub farm_id: String,
    pub region_id: u64,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub net_weekly_impact_assets: BigInt,
    #[serde(
        serialize_with = "crate::serde_utils::bigint_to_string",
        deserialize_with = "crate::serde_utils::de_bigint"
    )]
    pub total_protocol_deposit_value: BigInt,
    pub assets: Vec<AssetRequirement>,
    #[serde(default, rename = "rewardSplit")]
    pub reward_split: Vec<RewardSplit>,
    pub first_week: u64,
    pub weeks_alive: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputDataMultiAsset {
    #[serde(default, deserialize_with = "crate::serde_utils::de_leftovers_map")]
    pub cgp_leftovers: HashMap<u64, BigInt>,
    pub solar_farms: Vec<MultiAssetSolarFarm>,
    #[serde(default, deserialize_with = "crate::serde_utils::de_optional_gctl_map")]
    pub gctl_distribution: Option<HashMap<u64, BigInt>>,
    #[serde(default)]
    pub output_farms: Option<Vec<String>>,
}

// --- Multi-Asset Output Types ---

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRewardOut {
    pub asset_id: String,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub asset_earned: BigInt,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmRewardMultiAsset {
    pub id: String,
    pub farm_id: String,
    pub week_index: u64,
    pub region_id: u64,
    pub assets: Vec<AssetRewardOut>,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub glow_inflation_reward: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub protocol_deposit: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub expected_production: BigInt,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTraceMultiAsset {
    pub farm_id: String,
    pub asset_id: String,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub amount: BigInt,
    pub region_id: u64,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub inflation_reward_split_6_decimals: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub deposit_reward_split_6_decimals: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub glow_inflation_reward: BigInt,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletDistributionMultiAsset {
    pub user_address: String,
    pub assets_earned: BTreeMap<String, String>,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub glow_inflation_earned: BigInt,
    pub traces: Vec<WalletTraceMultiAsset>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionAssetSummary {
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub protocol_deposit_sum: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub carbon_credit_production_sum: BigInt,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicWeekOutputMultiAsset {
    pub wallet_distributions: Vec<WalletDistributionMultiAsset>,
    pub farm_rewards: Vec<FarmRewardMultiAsset>,
    pub region_data: BTreeMap<u64, BTreeMap<String, RegionAssetSummary>>,
    pub warnings: Vec<String>,
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
    let mut region_assets: HashMap<u64, HashSet<String>> = HashMap::new();
    for cid in comps.keys() {
        region_assets
            .entry(cid.region_id)
            .or_default()
            .insert(cid.asset_id.clone());
    }
    let total_regions = region_assets.len();
    let mut stats: Vec<RegionStats> = region_assets
        .into_iter()
        .map(|(region_id, assets)| {
            let mut assets_vec: Vec<String> = assets.into_iter().collect();
            assets_vec.sort();
            RegionStats {
                region_id,
                assets: assets_vec,
            }
        })
        .collect();
    stats.sort_by(|a, b| a.region_id.cmp(&b.region_id));
    (total_regions, stats)
}
