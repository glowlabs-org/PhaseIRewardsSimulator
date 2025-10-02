use crate::core_types::*;
use crate::errors::SimError;
use crate::models::*;
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};

const WEEK_BOUND: u64 = 1 << 12; // 4096
const MIN_WEEKS_ALIVE: u64 = 2;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationDiagnostics {
    pub output: OutputData,
    pub errors: Vec<String>,
    pub competitions: Vec<DetailedCompetition>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedCompetition {
    pub region_id: String,
    pub asset_id: String,
    pub first_week: u64,
    pub final_week: u64,
    pub farms: Vec<DetailedFarmInfo>,
    pub buckets: Vec<DetailedBucket>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedFarmInfo {
    pub farm_id: String,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub protocol_deposit_value: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub assets_required: BigInt,
    pub first_week: u64,
    pub final_week: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewards_address: Option<String>,
    pub asset_id: String,
    pub region_id: String,
    #[serde(rename = "rewardSplit")]
    pub reward_splits: Vec<RewardSplit>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedBucket {
    pub week_number: u64,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub total_deposits: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub total_impact_assets: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub pool_net_assets: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub pool_net_deposits: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub glw_inflation: BigInt,
    pub first_week_farms: Vec<String>,
    pub ongoing_farms: Vec<String>,
    pub last_week_farms: Vec<String>,
    pub farm_states: Vec<DetailedFarmBucketState>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedFarmBucketState {
    pub farm_id: String,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub deposits_contributed: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub impact_assets_contributed: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub accumulated_drawdown: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub net_overperformance: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub rewards_this_week: BigInt,
}

pub fn simulate(input: InputData) -> Result<OutputData, SimError> {
    let diag = simulate_with_diagnostics(input)?;
    if diag.errors.is_empty() {
        Ok(diag.output)
    } else {
        let joined = diag.errors.join(" | ");
        Err(SimError::algorithm(format!(
            "consistency issues detected: {joined}"
        )))
    }
}

fn tol_assets() -> BigInt {
    BigInt::from(1_000_000_000_000_000_000u128)
}
fn tol_dollars() -> BigInt {
    BigInt::from(1_000_000u64)
}

pub fn simulate_with_diagnostics(input: InputData) -> Result<SimulationDiagnostics, SimError> {
    validate_input(&input)?;
    let mut diagnostics: Vec<String> = Vec::new();
    let mut competitions: HashMap<CompetitionID, Competition> = HashMap::new();

    let mut seen_farms: HashSet<String> = HashSet::new();
    for farm in &input.solar_farms {
        if !seen_farms.insert(farm.farm_id.clone()) {
            let fid = &farm.farm_id;
            return Err(SimError::validation(format!("duplicate farm id: {fid}")));
        }
        let cid = CompetitionID {
            region_id: farm.region_id.clone(),
            asset_id: farm.asset_id.clone(),
        };
        let comp = competitions
            .entry(cid.clone())
            .or_insert_with(|| Competition {
                first_week: farm.first_week,
                final_week: farm.first_week + farm.weeks_alive - 1,
                buckets: HashMap::new(),
                farms: HashMap::new(),
            });

        comp.first_week = comp.first_week.min(farm.first_week);
        comp.final_week = comp.final_week.max(farm.first_week + farm.weeks_alive - 1);

        // Reward splits: use explicit splits if provided; otherwise, fallback to legacy rewards_address.
        let reward_splits = if !farm.reward_split.is_empty() {
            farm.reward_split.clone()
        } else if let Some(addr) = &farm.rewards_address {
            vec![RewardSplit {
                wallet_address: addr.clone(),
                glow_split_percent_6_decimals: BigInt::from(1_000_000u32),
                deposit_split_percent_6_decimals: BigInt::from(1_000_000u32),
            }]
        } else {
            Vec::new()
        };

        comp.farms.insert(
            farm.farm_id.clone(),
            FarmInfo {
                farm_id: farm.farm_id.clone(),
                protocol_deposit_value: farm.protocol_deposit_value.clone(),
                assets_required: farm.assets_required.clone(),
                first_week: farm.first_week,
                final_week: farm.first_week + farm.weeks_alive - 1,
                rewards_address: farm.rewards_address.clone(),
                asset_id: farm.asset_id.clone(),
                region_id: farm.region_id.clone(),
                reward_splits,
            },
        );

        let deposit_per = &farm.protocol_deposit_value / BigInt::from(farm.weeks_alive);
        let impact_assets_per = farm.weekly_impact_assets.clone();
        let final_week = farm.first_week + farm.weeks_alive - 1;

        for week in farm.first_week..=final_week {
            let bucket = comp.buckets.entry(week).or_insert_with(|| Bucket {
                total_deposits: BigInt::zero(),
                total_impact_assets: BigInt::zero(),
                first_week_farms: Vec::new(),
                ongoing_farms: Vec::new(),
                last_week_farms: Vec::new(),
                farm_states: HashMap::new(),
                pool_net_assets: BigInt::zero(),
                pool_net_deposits: BigInt::zero(),
                glw_inflation: BigInt::zero(),
            });

            bucket.total_deposits += &deposit_per;
            bucket.total_impact_assets += &impact_assets_per;

            if week == farm.first_week {
                bucket.first_week_farms.push(farm.farm_id.clone());
            } else if week == final_week {
                bucket.last_week_farms.push(farm.farm_id.clone());
            } else {
                bucket.ongoing_farms.push(farm.farm_id.clone());
            }

            bucket.farm_states.insert(
                farm.farm_id.clone(),
                FarmBucketState {
                    deposits_contributed: deposit_per.clone(),
                    impact_assets_contributed: impact_assets_per.clone(),
                    accumulated_drawdown: BigInt::zero(),
                    net_overperformance: BigInt::zero(),
                    rewards_this_week: BigInt::zero(),
                },
            );
        }
    }

    // Process competitions
    for (cid, comp) in competitions.iter_mut() {
        let mut weeks: Vec<u64> = comp.buckets.keys().copied().collect();
        weeks.sort_unstable();

        for (idx, week) in weeks.iter().enumerate() {
            let week = *week;
            let prev_is_immediate = if idx > 0 {
                let prev_week = weeks[idx - 1];
                prev_week + 1 == week
            } else {
                false
            };

            let prev_states: Option<HashMap<String, FarmBucketState>> = if prev_is_immediate {
                let prev_week = weeks[idx - 1];
                comp.buckets.get(&prev_week).map(|b| b.farm_states.clone())
            } else {
                None
            };

            // Determine pool carryover before taking a mutable borrow
            let (carry_assets, carry_deposits) = if prev_is_immediate {
                let prev_week = weeks[idx - 1];
                let prev_bucket = comp.buckets.get(&prev_week).expect("prev bucket exists");
                (
                    prev_bucket.pool_net_assets.clone(),
                    prev_bucket.pool_net_deposits.clone(),
                )
            } else {
                (BigInt::zero(), BigInt::zero())
            };

            let bucket = comp.buckets.get_mut(&week).expect("exists");

            // Pool state only carries across if weeks are consecutive; otherwise reset to zero
            bucket.pool_net_assets = carry_assets;
            bucket.pool_net_deposits = carry_deposits;

            if bucket.total_impact_assets.is_zero() {
                return Err(SimError::algorithm(format!(
                    "bucket at week {week} has zero total impact assets"
                )));
            }

            // Deterministic farm order
            let farm_order = farm_iter_order(bucket);

            // Stage 1: compute deposits_recovered and apply over/under-performance adjustments.
            let mut stage1_results: Vec<(String, FarmBucketState, BigInt)> =
                Vec::with_capacity(farm_order.len());
            for fid in &farm_order {
                let mut state = bucket
                    .farm_states
                    .get(fid)
                    .cloned()
                    .ok_or_else(|| SimError::internal("missing state"))?;

                // Farm state only carries across if previous week is immediate
                if prev_is_immediate {
                    if let Some(ref prev_map) = prev_states {
                        if let Some(prev_state) = prev_map.get(fid) {
                            state.accumulated_drawdown = prev_state.accumulated_drawdown.clone();
                            state.net_overperformance = prev_state.net_overperformance.clone();
                        }
                    }
                }

                let fmeta = comp
                    .farms
                    .get(fid)
                    .ok_or_else(|| SimError::internal("missing farm meta"))?;

                let deposits_recovered = (&state.impact_assets_contributed
                    * &bucket.total_deposits)
                    / &bucket.total_impact_assets;

                let delta = &deposits_recovered - &state.deposits_contributed;
                if delta >= BigInt::zero() {
                    state.net_overperformance += delta;
                } else {
                    let under = -delta;
                    if state.net_overperformance >= under {
                        state.net_overperformance -= &under;
                    } else {
                        let penalty_drawdown = &under - &state.net_overperformance;
                        state.net_overperformance = BigInt::zero();
                        state.accumulated_drawdown += &penalty_drawdown;
                        let pen_assets = (&penalty_drawdown * &fmeta.assets_required)
                            / &fmeta.protocol_deposit_value;
                        bucket.pool_net_deposits += &penalty_drawdown;
                        bucket.pool_net_assets += pen_assets;
                    }
                }

                stage1_results.push((fid.clone(), state, deposits_recovered));
            }

            // Stage 2: compute rewards and pool withdrawals with the pool fully funded from Stage 1.
            for (fid, mut state, deposits_recovered) in stage1_results {
                let fmeta = comp
                    .farms
                    .get(&fid)
                    .ok_or_else(|| SimError::internal("missing farm meta"))?;

                let mut rewards = BigInt::zero();
                let mut remaining = deposits_recovered.clone();

                // Collect from own vault first
                if state.accumulated_drawdown < fmeta.protocol_deposit_value {
                    let capacity = &fmeta.protocol_deposit_value - &state.accumulated_drawdown;
                    let take_own = min_bigint(&remaining, &capacity);
                    if !take_own.is_zero() {
                        let own_assets =
                            (&take_own * &fmeta.assets_required) / &fmeta.protocol_deposit_value;
                        rewards += &own_assets;
                        state.accumulated_drawdown += &take_own;
                        remaining -= take_own;
                    }
                }

                // Then collect from pool
                if !remaining.is_zero() && !bucket.pool_net_deposits.is_zero() {
                    let pool_take_limit = bucket.pool_net_deposits.clone();
                    let over_lim = state.net_overperformance.clone();
                    let take_pool =
                        min_bigint(&remaining, &min_bigint(&pool_take_limit, &over_lim));

                    if !take_pool.is_zero() {
                        let assets_from_pool =
                            (&take_pool * &bucket.pool_net_assets) / &bucket.pool_net_deposits;
                        rewards += &assets_from_pool;
                        state.net_overperformance -= &take_pool;
                        bucket.pool_net_deposits -= &take_pool;
                        bucket.pool_net_assets -= &assets_from_pool;
                        remaining -= take_pool;
                    }
                }

                // CGP leftovers bonus
                if cid.region_id == "cgp" && cid.asset_id == "usdg" {
                    if let Some(leftover) = input.cgp_leftovers.get(&week) {
                        if !bucket.total_deposits.is_zero() {
                            let bonus = (&deposits_recovered * leftover) / &bucket.total_deposits;
                            rewards += bonus;
                        }
                    }
                }

                state.rewards_this_week = rewards;
                bucket.farm_states.insert(fid.clone(), state.clone());

                // Final-week consistency checks for the farm (dollar-denominated)
                if week == fmeta.final_week {
                    let st = &bucket.farm_states[&fid];
                    let diff = (&st.accumulated_drawdown - &fmeta.protocol_deposit_value).abs();

                    let farm_tolerance = tol_dollars();

                    if diff > farm_tolerance {
                        let acc = &st.accumulated_drawdown;
                        let expected = &fmeta.protocol_deposit_value;
                        let tol = &farm_tolerance;
                        diagnostics.push(format!(
                            "final-week drawdown mismatch for farm {fid} week {week}: accumulated_drawdown={acc}, expected_protocol_deposit_value={expected}, tolerance={tol}"
                        ));
                    }

                    let over_abs = st.net_overperformance.abs();
                    if over_abs > farm_tolerance {
                        let nop = &st.net_overperformance;
                        diagnostics.push(format!(
                            "final-week overperformance not near zero for farm {fid} week {week}: net_overperformance={nop}, tolerance={farm_tolerance}"
                        ));
                    }
                }
            }

            // Gap-boundary checks
            let next_is_immediate = if idx + 1 < weeks.len() {
                let next_week = weeks[idx + 1];
                next_week == week + 1
            } else {
                false
            };
            if !next_is_immediate {
                let region = &cid.region_id;
                let asset = &cid.asset_id;
                let net_deposits = &bucket.pool_net_deposits;
                let net_assets = &bucket.pool_net_assets;

                let tol_dep = tol_dollars();
                let tol_ast = tol_assets();

                let ok_assets = bucket.pool_net_assets.abs() <= tol_ast;
                let ok_deposits = bucket.pool_net_deposits.abs() <= tol_dep;

                if !ok_assets || !ok_deposits {
                    diagnostics.push(format!(
                        "pool not settled at gap boundary for region={region} asset={asset} at week={week}: net_deposits={net_deposits}, net_assets={net_assets}, tolerances(dollars={tol_dep}, assets={tol_ast})"
                    ));
                }
            }
        }
    }

    // Apply GCTL GLW inflation to buckets after simulation
    crate::gctl::apply_gctl_inflation(&mut competitions);

    // Build legacy output for tests
    let (total_regions, regional_stats) = unique_regions_and_assets(&competitions);

    let mut global_first = u64::MAX;
    let mut global_last = 0u64;
    for comp in competitions.values() {
        global_first = global_first.min(comp.first_week);
        global_last = global_last.max(comp.final_week);
    }

    let mut weekly_rewards = Vec::new();
    if global_first != u64::MAX {
        for week in global_first..=global_last {
            let mut per_farm = Vec::<FarmReward>::new();
            for comp in competitions.values() {
                if let Some(bucket) = comp.buckets.get(&week) {
                    for fid in farm_iter_order(bucket) {
                        let st = &bucket.farm_states[&fid];
                        let finfo = &comp.farms[&fid];
                        per_farm.push(FarmReward {
                            farm_id: fid.clone(),
                            asset_id: finfo.asset_id.clone(),
                            region_id: finfo.region_id.clone(),
                            amount: st.rewards_this_week.clone(),
                            rewards_address: finfo.rewards_address.clone(),
                        });
                    }
                }
            }
            if !per_farm.is_empty() {
                weekly_rewards.push(WeekRewards {
                    week_number: week,
                    per_farm_rewards: per_farm,
                });
            }
        }
    }

    let output = OutputData {
        total_regions,
        regional_stats,
        weekly_rewards,
    };

    // Build detailed snapshot of internal state for diagnostics endpoint (include reward splits)
    let competitions_detailed = build_detailed_competitions(&competitions);

    Ok(SimulationDiagnostics {
        output,
        errors: diagnostics,
        competitions: competitions_detailed,
    })
}

fn build_detailed_competitions(
    competitions: &HashMap<CompetitionID, Competition>,
) -> Vec<DetailedCompetition> {
    // Sort competitions deterministically by (region_id, asset_id)
    let mut items: Vec<(&CompetitionID, &Competition)> = competitions.iter().collect();
    items.sort_by(|(a_id, _), (b_id, _)| {
        let r = a_id.region_id.cmp(&b_id.region_id);
        if r == std::cmp::Ordering::Equal {
            a_id.asset_id.cmp(&b_id.asset_id)
        } else {
            r
        }
    });

    let mut out = Vec::with_capacity(items.len());
    for (cid, comp) in items {
        // Farms sorted by farm_id for determinism
        let mut farms_vec: Vec<DetailedFarmInfo> = comp
            .farms
            .values()
            .map(|f| DetailedFarmInfo {
                farm_id: f.farm_id.clone(),
                protocol_deposit_value: f.protocol_deposit_value.clone(),
                assets_required: f.assets_required.clone(),
                first_week: f.first_week,
                final_week: f.final_week,
                rewards_address: f.rewards_address.clone(),
                asset_id: f.asset_id.clone(),
                region_id: f.region_id.clone(),
                reward_splits: f.reward_splits.clone(),
            })
            .collect();
        farms_vec.sort_by(|a, b| a.farm_id.cmp(&b.farm_id));

        // Buckets sorted by week, with deterministic farm order inside
        let mut week_keys: Vec<u64> = comp.buckets.keys().copied().collect();
        week_keys.sort_unstable();
        let mut buckets_vec: Vec<DetailedBucket> = Vec::with_capacity(week_keys.len());
        for w in week_keys {
            let b = &comp.buckets[&w];
            let farm_states_vec: Vec<DetailedFarmBucketState> = farm_iter_order(b)
                .into_iter()
                .map(|fid| {
                    let st = &b.farm_states[&fid];
                    DetailedFarmBucketState {
                        farm_id: fid,
                        deposits_contributed: st.deposits_contributed.clone(),
                        impact_assets_contributed: st.impact_assets_contributed.clone(),
                        accumulated_drawdown: st.accumulated_drawdown.clone(),
                        net_overperformance: st.net_overperformance.clone(),
                        rewards_this_week: st.rewards_this_week.clone(),
                    }
                })
                .collect();

            buckets_vec.push(DetailedBucket {
                week_number: w,
                total_deposits: b.total_deposits.clone(),
                total_impact_assets: b.total_impact_assets.clone(),
                pool_net_assets: b.pool_net_assets.clone(),
                pool_net_deposits: b.pool_net_deposits.clone(),
                glw_inflation: b.glw_inflation.clone(),
                first_week_farms: b.first_week_farms.clone(),
                ongoing_farms: b.ongoing_farms.clone(),
                last_week_farms: b.last_week_farms.clone(),
                farm_states: farm_states_vec,
            });
        }

        out.push(DetailedCompetition {
            region_id: cid.region_id.clone(),
            asset_id: cid.asset_id.clone(),
            first_week: comp.first_week,
            final_week: comp.final_week,
            farms: farms_vec,
            buckets: buckets_vec,
        });
    }
    out
}

fn validate_input(input: &InputData) -> Result<(), SimError> {
    if input.solar_farms.is_empty() {
        return Err(SimError::validation("no farms provided"));
    }
    for f in &input.solar_farms {
        if f.farm_id.trim().is_empty() {
            return Err(SimError::validation("empty farm_id"));
        }
        if f.asset_id.trim().is_empty() || f.region_id.trim().is_empty() {
            return Err(SimError::validation("empty asset_id/region_id"));
        }
        if let Some(addr) = &f.rewards_address {
            if !addr.trim().is_empty() && !is_valid_eth_address(addr) {
                return Err(SimError::validation(format!(
                    "invalid rewards_address: {addr}"
                )));
            }
        }
        if f.first_week == 0 || f.first_week >= WEEK_BOUND {
            return Err(SimError::validation(format!(
                "first_week must be > 0 and < {WEEK_BOUND}"
            )));
        }
        if f.weeks_alive < MIN_WEEKS_ALIVE || f.weeks_alive > WEEK_BOUND {
            return Err(SimError::validation(format!(
                "weeks_alive must be >= {MIN_WEEKS_ALIVE} and <= {WEEK_BOUND}"
            )));
        }
        if f.weekly_impact_assets < BigInt::zero() {
            return Err(SimError::validation(
                "weekly_impact_assets cannot be negative",
            ));
        }
        if f.protocol_deposit_value <= BigInt::zero() || f.assets_required <= BigInt::zero() {
            return Err(SimError::validation(
                "protocol_deposit_value and assets_required must be positive",
            ));
        }

        // Reward split invariants if provided
        if !f.reward_split.is_empty() {
            let mut glow_sum = BigInt::zero();
            let mut dep_sum = BigInt::zero();
            let million = BigInt::from(1_000_000u32);
            for sp in &f.reward_split {
                if !is_valid_eth_address(&sp.wallet_address) {
                    return Err(SimError::validation(format!(
                        "invalid rewardSplit walletAddress: {}",
                        sp.wallet_address
                    )));
                }
                if sp.glow_split_percent_6_decimals < BigInt::zero()
                    || sp.deposit_split_percent_6_decimals < BigInt::zero()
                {
                    return Err(SimError::validation(
                        "rewardSplit percents cannot be negative",
                    ));
                }
                if sp.glow_split_percent_6_decimals > million
                    || sp.deposit_split_percent_6_decimals > million
                {
                    return Err(SimError::validation(
                        "rewardSplit percents cannot exceed 1000000",
                    ));
                }
                glow_sum += sp.glow_split_percent_6_decimals.clone();
                dep_sum += sp.deposit_split_percent_6_decimals.clone();
            }
            if glow_sum != million {
                return Err(SimError::validation(format!(
                    "rewardSplit glowSplitPercent6Decimals must sum to 1000000 for farm {}",
                    f.farm_id
                )));
            }
            if dep_sum != million {
                return Err(SimError::validation(format!(
                    "rewardSplit depositSplitPercent6Decimals must sum to 1000000 for farm {}",
                    f.farm_id
                )));
            }
        }
    }
    Ok(())
}

fn farm_iter_order(bucket: &Bucket) -> Vec<String> {
    let mut order = Vec::with_capacity(
        bucket.first_week_farms.len() + bucket.ongoing_farms.len() + bucket.last_week_farms.len(),
    );
    order.extend(bucket.first_week_farms.iter().cloned());
    order.extend(bucket.ongoing_farms.iter().cloned());
    order.extend(bucket.last_week_farms.iter().cloned());
    order
}

fn min_bigint(a: &BigInt, b: &BigInt) -> BigInt {
    if a <= b {
        a.clone()
    } else {
        b.clone()
    }
}

// ------------------------------
// Region key serialization helper for public JSON
// ------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionKey {
    Known(u64),
    Other(String),
}

impl RegionKey {
    fn from_str(s: &str) -> Self {
        match s {
            "cgp" => RegionKey::Known(1),
            "utah" => RegionKey::Known(2),
            other => RegionKey::Other(other.to_string()),
        }
    }
}

impl Ord for RegionKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (RegionKey::Known(a), RegionKey::Known(b)) => a.cmp(b),
            (RegionKey::Known(_), RegionKey::Other(_)) => Ordering::Less,
            (RegionKey::Other(_), RegionKey::Known(_)) => Ordering::Greater,
            (RegionKey::Other(a), RegionKey::Other(b)) => a.cmp(b),
        }
    }
}
impl PartialOrd for RegionKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for RegionKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RegionKey::Known(n) => serializer.serialize_u64(*n),
            RegionKey::Other(s) => serializer.serialize_str(s),
        }
    }
}

// ------------------------------
// Public output composition
// ------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTrace {
    pub farm_id: String,
    pub asset: String,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub inflation_reward_split_6_decimals: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub deposit_reward_split_6_decimals: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub amount: BigInt,
    pub region_id: RegionKey,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub glow_inflation_reward: BigInt,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletDistribution {
    pub user_address: String,
    // Map<asset_id, amount_as_string>
    pub assets_earned: BTreeMap<String, String>,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub glow_inflation_earned: BigInt,
    pub traces: Vec<WalletTrace>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FarmRewardOut {
    pub id: String,
    pub asset: String,
    pub region_id: RegionKey,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub asset_earned: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub glow_inflation_reward: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub protocol_deposit: BigInt,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub expected_production: BigInt,
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
pub struct PublicWeekOutput {
    pub wallet_distributions: Vec<WalletDistribution>,
    pub farm_rewards: Vec<FarmRewardOut>,
    pub region_data: BTreeMap<RegionKey, BTreeMap<String, RegionAssetSummary>>,
    pub warnings: Vec<String>,
}

pub fn build_public_output_from_detailed(
    competitions: &[DetailedCompetition],
    warnings: &[String],
) -> BTreeMap<String, PublicWeekOutput> {
    // week -> aggregator
    struct Agg {
        farm_rewards: Vec<FarmRewardOut>,
        region_data: BTreeMap<RegionKey, BTreeMap<String, RegionAssetSummary>>,
        wallets: BTreeMap<String, WalletDistribution>,
    }
    let mut by_week: BTreeMap<u64, Agg> = BTreeMap::new();

    for comp in competitions {
        // Build meta map for quick lookup
        let mut meta: HashMap<String, &DetailedFarmInfo> = HashMap::new();
        for f in &comp.farms {
            meta.insert(f.farm_id.clone(), f);
        }

        for b in &comp.buckets {
            let entry = by_week.entry(b.week_number).or_insert_with(|| Agg {
                farm_rewards: Vec::new(),
                region_data: BTreeMap::new(),
                wallets: BTreeMap::new(),
            });

            // region data
            let rkey = RegionKey::from_str(&comp.region_id);
            let reg_map = entry.region_data.entry(rkey).or_default();
            let sums = reg_map
                .entry(comp.asset_id.clone())
                .or_insert(RegionAssetSummary {
                    protocol_deposit_sum: BigInt::zero(),
                    carbon_credit_production_sum: BigInt::zero(),
                });
            sums.protocol_deposit_sum += b.total_deposits.clone();
            sums.carbon_credit_production_sum += b.total_impact_assets.clone();

            // farm rewards and wallet distributions
            for st in &b.farm_states {
                let Some(finfo) = meta.get(&st.farm_id) else {
                    continue;
                };

                let glw_per_farm = if b.total_deposits.is_zero() {
                    BigInt::zero()
                } else {
                    (&b.glw_inflation * &st.deposits_contributed) / &b.total_deposits
                };

                entry.farm_rewards.push(FarmRewardOut {
                    id: st.farm_id.clone(),
                    asset: finfo.asset_id.clone(),
                    region_id: RegionKey::from_str(&finfo.region_id),
                    asset_earned: st.rewards_this_week.clone(),
                    glow_inflation_reward: glw_per_farm.clone(),
                    protocol_deposit: finfo.protocol_deposit_value.clone(),
                    // expectedProduction is alias for netWeeklyImpactAssets (per-week)
                    expected_production: st.impact_assets_contributed.clone(),
                });

                // apply reward splits
                if !finfo.reward_splits.is_empty() {
                    let million = BigInt::from(1_000_000u32);
                    for sp in &finfo.reward_splits {
                        let asset_part = (&st.rewards_this_week
                            * &sp.deposit_split_percent_6_decimals)
                            / &million;
                        let glw_part =
                            (&glw_per_farm * &sp.glow_split_percent_6_decimals) / &million;

                        let addr_key = sp.wallet_address.clone();
                        let wallet = entry.wallets.entry(addr_key.clone()).or_insert_with(|| {
                            WalletDistribution {
                                user_address: addr_key.clone(),
                                assets_earned: BTreeMap::new(),
                                glow_inflation_earned: BigInt::zero(),
                                traces: Vec::new(),
                            }
                        });

                        let ae = wallet
                            .assets_earned
                            .entry(finfo.asset_id.clone())
                            .or_insert_with(|| "0".to_string());
                        let cur = ae.parse::<BigInt>().unwrap_or_else(|_| BigInt::zero());
                        let new = cur + asset_part.clone();
                        *ae = new.to_string();

                        wallet.glow_inflation_earned += glw_part.clone();

                        wallet.traces.push(WalletTrace {
                            farm_id: st.farm_id.clone(),
                            asset: finfo.asset_id.clone(),
                            inflation_reward_split_6_decimals: sp
                                .glow_split_percent_6_decimals
                                .clone(),
                            deposit_reward_split_6_decimals: sp
                                .deposit_split_percent_6_decimals
                                .clone(),
                            amount: asset_part,
                            region_id: RegionKey::from_str(&finfo.region_id),
                            glow_inflation_reward: glw_part,
                        });
                    }
                } else if let Some(addr) = &finfo.rewards_address {
                    // Legacy fallback: 100% to rewards_address
                    let glw_share = glw_per_farm.clone();
                    let asset_share = st.rewards_this_week.clone();
                    let addr_key = addr.clone();
                    let wallet = entry.wallets.entry(addr_key.clone()).or_insert_with(|| {
                        WalletDistribution {
                            user_address: addr_key.clone(),
                            assets_earned: BTreeMap::new(),
                            glow_inflation_earned: BigInt::zero(),
                            traces: Vec::new(),
                        }
                    });
                    // merge asset
                    let ae = wallet
                        .assets_earned
                        .entry(finfo.asset_id.clone())
                        .or_insert_with(|| "0".to_string());
                    let cur = ae.parse::<BigInt>().unwrap_or_else(|_| BigInt::zero());
                    let new = cur + asset_share.clone();
                    *ae = new.to_string();

                    wallet.glow_inflation_earned += glw_share.clone();

                    wallet.traces.push(WalletTrace {
                        farm_id: st.farm_id.clone(),
                        asset: finfo.asset_id.clone(),
                        inflation_reward_split_6_decimals: BigInt::from(1_000_000u32),
                        deposit_reward_split_6_decimals: BigInt::from(1_000_000u32),
                        amount: asset_share,
                        region_id: RegionKey::from_str(&finfo.region_id),
                        glow_inflation_reward: glw_share,
                    });
                } else {
                    // No address and no explicit splits: nothing to attribute at wallet level.
                }
            }
        }
    }

    // finalize with warnings attached to each week
    let mut out: BTreeMap<String, PublicWeekOutput> = BTreeMap::new();
    for (w, agg) in by_week {
        let mut wd: Vec<WalletDistribution> = agg.wallets.into_values().collect();
        wd.sort_by(|a, b| a.user_address.cmp(&b.user_address));
        let mut fr = agg.farm_rewards;
        fr.sort_by(|a, b| a.id.cmp(&b.id));

        out.insert(
            w.to_string(),
            PublicWeekOutput {
                wallet_distributions: wd,
                farm_rewards: fr,
                region_data: agg.region_data,
                warnings: warnings.to_vec(),
            },
        );
    }
    out
}
