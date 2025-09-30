use crate::core_types::*;
use crate::errors::SimError;
use crate::models::*;
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

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

                // CGP leftovers (bonus, independent of vault/pool accounting)
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

                // Final-week consistency checks for the farm with generous tolerance
                if week == fmeta.final_week {
                    let st = &bucket.farm_states[&fid];
                    let diff = (&st.accumulated_drawdown - &fmeta.protocol_deposit_value).abs();

                    let farm_tolerance = BigInt::from(1_000_000_000u64);

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

            // Gap-boundary checks: if no immediate next bucket, pool should be near zero
            let tolerance = BigInt::from(1_000_000_000u64);
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
                let ok_assets = bucket.pool_net_assets.abs() <= tolerance;
                let ok_deposits = bucket.pool_net_deposits.abs() <= tolerance;
                if !ok_assets || !ok_deposits {
                    let tol = &tolerance;
                    diagnostics.push(format!(
                        "pool not settled at gap boundary for region={region} asset={asset} at week={week}: net_deposits={net_deposits}, net_assets={net_assets}, tolerance={tol}"
                    ));
                }
            }
        }
    }

    // Build output per spec: determine global first/last week, iterate and collect
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

    // Build detailed snapshot of internal state for diagnostics endpoint
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
                "weekly_impact_assets must be positive",
            ));
        }
        if f.protocol_deposit_value <= BigInt::zero() || f.assets_required <= BigInt::zero() {
            return Err(SimError::validation(
                "protocol_deposit_value and assets_required must be positive",
            ));
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
