use crate::errors::SimError;
use crate::models::*;
use num_bigint::BigInt;
use num_traits::{One, Zero};
use std::collections::{HashMap, HashSet};

const WEEK_BOUND: u64 = 1 << 12;
const MIN_WEEKS_ALIVE: u64 = 2;

pub fn simulate(input: InputData) -> Result<OutputData, SimError> {
    validate_input(&input)?;
    let mut competitions: HashMap<CompetitionID, Competition> = HashMap::new();

    let mut seen_farms: HashSet<String> = HashSet::new();
    for farm in &input.solar_farms {
        if !seen_farms.insert(farm.farm_id.clone()) {
            return Err(SimError::validation(format!(
                "duplicate farm id: {}",
                farm.farm_id
            )));
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
        let cc_per = farm.weekly_carbon_credits.clone();
        let final_week = farm.first_week + farm.weeks_alive - 1;

        for week in farm.first_week..=final_week {
            let bucket = comp.buckets.entry(week).or_insert_with(|| Bucket {
                total_deposits: BigInt::zero(),
                total_carbon_credits: BigInt::zero(),
                first_week_farms: Vec::new(),
                ongoing_farms: Vec::new(),
                last_week_farms: Vec::new(),
                farm_states: HashMap::new(),
                pool_net_assets: BigInt::zero(),
                pool_net_deposits: BigInt::zero(),
            });

            bucket.total_deposits += &deposit_per;
            bucket.total_carbon_credits += &cc_per;

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
                    carbon_credits_contributed: cc_per.clone(),
                    accumulated_drawdown: BigInt::zero(),
                    net_overperformance: BigInt::zero(),
                    rewards_this_week: BigInt::zero(),
                },
            );
        }
    }

    // Process competitions
    for (cid, comp) in competitions.iter_mut() {
        let mut prev_pool_assets = BigInt::zero();
        let mut prev_pool_deposits = BigInt::zero();

        let mut prev_bucket_week: Option<u64> = None;

        let mut weeks: Vec<u64> = comp.buckets.keys().copied().collect();
        weeks.sort_unstable();
        for week in weeks.clone() {
            let prev_states: Option<HashMap<String, FarmBucketState>> = prev_bucket_week
                .and_then(|prev_wk| comp.buckets.get(&prev_wk).map(|b| b.farm_states.clone()));

            let bucket = comp.buckets.get_mut(&week).expect("exists");
            if prev_bucket_week.is_some() {
                bucket.pool_net_assets = prev_pool_assets.clone();
                bucket.pool_net_deposits = prev_pool_deposits.clone();
            } else {
                bucket.pool_net_assets = BigInt::zero();
                bucket.pool_net_deposits = BigInt::zero();
            }

            if bucket.total_carbon_credits.is_zero() {
                return Err(SimError::algorithm(format!(
                    "bucket at week {week} has zero total carbon credits"
                )));
            }

            // Deterministic farm order
            let farm_order = farm_iter_order(bucket);

            // Stage 1: compute deposits_recovered and apply over/under-performance adjustments.
            // All penalty contributions are added to the pool before any withdrawals happen,
            // eliminating order-dependent residuals within a bucket.
            let mut stage1_results: Vec<(String, FarmBucketState, BigInt)> =
                Vec::with_capacity(farm_order.len());
            for fid in &farm_order {
                let mut state = bucket
                    .farm_states
                    .get(fid)
                    .cloned()
                    .ok_or_else(|| SimError::internal("missing state"))?;

                if let Some(ref prev_map) = prev_states {
                    if let Some(prev_state) = prev_map.get(fid) {
                        state.accumulated_drawdown = prev_state.accumulated_drawdown.clone();
                        state.net_overperformance = prev_state.net_overperformance.clone();
                    }
                }

                let fmeta = comp
                    .farms
                    .get(fid)
                    .ok_or_else(|| SimError::internal("missing farm meta"))?;

                let deposits_recovered = (&state.carbon_credits_contributed
                    * &bucket.total_deposits)
                    / &bucket.total_carbon_credits;

                // over/under-performance delta relative to contribution
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

            // Stage 2: compute rewards and pool withdrawals with the pool fully funded
            // from Stage 1.
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

                // Final-week consistency checks for the farm (allowing 1 unit of dust)
                if week == fmeta.final_week {
                    let st = &bucket.farm_states[&fid];
                    let diff = (&st.accumulated_drawdown - &fmeta.protocol_deposit_value).abs();
                    if diff > BigInt::one() {
                        return Err(SimError::algorithm(format!(
                            "final-week drawdown mismatch for farm {} week {}: accumulated_drawdown={}, expected_protocol_deposit_value={}",
                            fid, week, st.accumulated_drawdown, fmeta.protocol_deposit_value
                        )));
                    }
                    if st.net_overperformance < BigInt::zero()
                        || st.net_overperformance > BigInt::one()
                    {
                        return Err(SimError::algorithm(format!(
                            "final-week overperformance not near zero for farm {} week {}: net_overperformance={}",
                            fid, week, st.net_overperformance
                        )));
                    }
                }
            }

            prev_pool_assets = bucket.pool_net_assets.clone();
            prev_pool_deposits = bucket.pool_net_deposits.clone();
            prev_bucket_week = Some(week);
        }

        // Allow some deterministic dust at the competition level:
        // tolerance = max(1, number of buckets in this competition)
        let tolerance = BigInt::from(comp.buckets.len() as u64).max(BigInt::one());

        if let Some((last_week, last_bucket)) = comp.buckets.iter().max_by_key(|(w, _)| *w) {
            let ok_assets = last_bucket.pool_net_assets.abs() <= tolerance;
            let ok_deposits = last_bucket.pool_net_deposits.abs() <= tolerance;
            if !ok_assets || !ok_deposits {
                return Err(SimError::algorithm(format!(
                    "competition pool not settled for region={} asset={} at final_week={}: net_deposits={}, net_assets={}, tolerance={}",
                    cid.region_id, cid.asset_id, last_week, last_bucket.pool_net_deposits, last_bucket.pool_net_assets, tolerance
                )));
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

    Ok(OutputData {
        total_regions,
        regional_stats,
        weekly_rewards,
    })
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
        if !is_valid_eth_address(&f.rewards_address) {
            return Err(SimError::validation(format!(
                "invalid rewards_address: {}",
                f.rewards_address
            )));
        }
        if f.first_week == 0
            || f.first_week >= WEEK_BOUND
            || f.weeks_alive < MIN_WEEKS_ALIVE
            || f.weeks_alive >= WEEK_BOUND
        {
            return Err(SimError::validation(format!(
                "first_week must be > 0 and < {WEEK_BOUND}; weeks_alive must be >= {MIN_WEEKS_ALIVE} and < {WEEK_BOUND}"
            )));
        }
        if f.weekly_carbon_credits <= BigInt::zero() {
            return Err(SimError::validation(
                "weekly_carbon_credits must be positive",
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
