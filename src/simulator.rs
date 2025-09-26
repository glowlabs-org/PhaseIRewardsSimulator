use crate::errors::SimError;
use crate::models::*;
use num_bigint::BigInt;
use num_traits::Zero;
use std::collections::{BTreeMap, HashMap, HashSet};

pub fn simulate(input: InputData) -> Result<OutputData, SimError> {
    validate_input(&input)?;
    let mut competitions: BTreeMap<CompetitionID, Competition> = BTreeMap::new();

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
                buckets: BTreeMap::new(),
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

        let weeks: Vec<u64> = comp.buckets.keys().copied().collect();
        for week in weeks.iter() {
            // Snapshot previous bucket farm states to avoid aliasing comp.buckets while holding a mutable borrow.
            let prev_states: Option<HashMap<String, FarmBucketState>> =
                if let Some(prev_wk) = prev_bucket_week {
                    comp.buckets.get(&prev_wk).map(|b| b.farm_states.clone())
                } else {
                    None
                };

            let bucket = comp.buckets.get_mut(week).expect("exists");
            // carry over pool state
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

            let farm_order = farm_iter_order(bucket);
            for fid in farm_order {
                let mut state = bucket
                    .farm_states
                    .get(&fid)
                    .cloned()
                    .ok_or_else(|| SimError::Internal("missing state".into()))?;

                // bring over previous farm state if present in previous week
                if let Some(ref prev_map) = prev_states {
                    if let Some(prev_state) = prev_map.get(&fid) {
                        state.accumulated_drawdown = prev_state.accumulated_drawdown.clone();
                        state.net_overperformance = prev_state.net_overperformance.clone();
                    }
                }

                let fmeta = comp
                    .farms
                    .get(&fid)
                    .ok_or_else(|| SimError::Internal("missing farm meta".into()))?;

                // deposits recovered
                let deposits_recovered = (&state.carbon_credits_contributed
                    * &bucket.total_deposits)
                    / &bucket.total_carbon_credits;

                // update over/under performance
                let delta = &deposits_recovered - &state.deposits_contributed;
                if delta.sign() == num_bigint::Sign::Plus || delta.is_zero() {
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
                        bucket.pool_net_deposits += penalty_drawdown;
                        bucket.pool_net_assets += pen_assets;
                    }
                }

                // rewards: own vault then pool
                let mut rewards = BigInt::zero();
                let mut remaining = deposits_recovered.clone();

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

                if !remaining.is_zero() {
                    // limited by pool deposits and by net_overperformance
                    let pool_take_limit = bucket.pool_net_deposits.clone();
                    let over_lim = state.net_overperformance.clone();
                    let take_pool =
                        min_bigint(&remaining, &min_bigint(&pool_take_limit, &over_lim));
                    if !take_pool.is_zero() && !bucket.pool_net_deposits.is_zero() {
                        let ratio = &bucket.pool_net_assets / &bucket.pool_net_deposits;
                        let pool_assets = &take_pool * ratio;
                        rewards += &pool_assets;
                        state.net_overperformance -= &take_pool;
                        bucket.pool_net_deposits -= &take_pool;
                        bucket.pool_net_assets -= pool_assets;
                        remaining -= take_pool;
                    }
                }

                // CGP leftovers bonus
                if cid.region_id == "cgp" && cid.asset_id == "usdg" {
                    if let Some(leftover) = input.cgp_leftovers.get(week) {
                        if !bucket.total_deposits.is_zero() {
                            let bonus = (&deposits_recovered * leftover) / &bucket.total_deposits;
                            rewards += bonus;
                        }
                    }
                }

                state.rewards_this_week = rewards;
                bucket.farm_states.insert(fid.clone(), state.clone());

                // final-week consistency
                if *week == fmeta.final_week {
                    let st = &bucket.farm_states[&fid];
                    if st.accumulated_drawdown != fmeta.protocol_deposit_value {
                        return Err(SimError::algorithm(format!(
                            "final-week drawdown mismatch for farm {fid} week {week}"
                        )));
                    }
                    if !st.net_overperformance.is_zero() {
                        return Err(SimError::algorithm(format!(
                            "final-week overperformance not zero for farm {fid} week {week}"
                        )));
                    }
                }
            }

            prev_pool_assets = bucket.pool_net_assets.clone();
            prev_pool_deposits = bucket.pool_net_deposits.clone();
            prev_bucket_week = Some(*week);
        }

        // Competition-level pool must end at zero
        if let Some((_, last_bucket)) = comp.buckets.iter().next_back() {
            if !last_bucket.pool_net_assets.is_zero() || !last_bucket.pool_net_deposits.is_zero() {
                return Err(SimError::algorithm(format!(
                    "competition pool not settled for region {} asset {}",
                    cid.region_id, cid.asset_id
                )));
            }
        }
    }

    // Build output
    let (total_regions, regional_stats) = unique_regions_and_assets(&competitions);
    let mut all_weeks = BTreeMap::<u64, Vec<FarmReward>>::new();
    for comp in competitions.values() {
        for (week, bucket) in comp.buckets.iter() {
            let mut rewards = Vec::new();
            for fid in farm_iter_order(bucket) {
                let st = &bucket.farm_states[&fid];
                let finfo = &comp.farms[&fid];
                rewards.push(FarmReward {
                    farm_id: fid.clone(),
                    asset_id: finfo.asset_id.clone(),
                    region_id: finfo.region_id.clone(),
                    amount: st.rewards_this_week.clone(),
                    rewards_address: finfo.rewards_address.clone(),
                });
            }
            all_weeks.entry(*week).or_default().extend(rewards);
        }
    }
    let mut weekly_rewards = Vec::new();
    for (week, list) in all_weeks {
        if !list.is_empty() {
            weekly_rewards.push(WeekRewards {
                week_number: week,
                per_farm_rewards: list,
            });
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
            || f.weeks_alive == 0
            || f.first_week >= (1 << 12)
            || f.weeks_alive >= (1 << 12)
        {
            return Err(SimError::validation(
                "first_week and weeks_alive must be >0 and < 4096",
            ));
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

#[cfg(test)]
mod unit {
    use super::*;
    use num_traits::FromPrimitive;
    use num_traits::One;

    #[test]
    fn eth_address_validation() {
        assert!(is_valid_eth_address(
            "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D"
        ));
        assert!(!is_valid_eth_address("0x123"));
        assert!(!is_valid_eth_address(
            "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
        ));
        assert!(!is_valid_eth_address(
            "6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D"
        ));
    }

    #[test]
    fn basic_build_and_simulate() {
        let input = InputData {
            cgp_leftovers: HashMap::new(),
            solar_farms: vec![
                SolarFarm {
                    farm_id: "A".into(),
                    asset_id: "glw".into(),
                    region_id: "cgp".into(),
                    weekly_carbon_credits: BigInt::one(),
                    protocol_deposit_value: BigInt::from_u64(10000).unwrap(),
                    assets_required: BigInt::from_u64(20000).unwrap(), // 2 per unit
                    rewards_address: "0x6Fbd1b5015deb91Dde137fc549dF1D04E09eAb6D".into(),
                    first_week: 10,
                    weeks_alive: 2,
                },
                SolarFarm {
                    farm_id: "B".into(),
                    asset_id: "glw".into(),
                    region_id: "cgp".into(),
                    weekly_carbon_credits: BigInt::one(),
                    protocol_deposit_value: BigInt::from_u64(10000).unwrap(),
                    assets_required: BigInt::from_u64(20000).unwrap(),
                    rewards_address: "0xa273164a466dbF9F0173996078fb382acC73F9E3".into(),
                    first_week: 10,
                    weeks_alive: 2,
                },
            ],
        };
        let out = simulate(input).expect("ok");
        assert_eq!(out.total_regions, 1);
        assert_eq!(out.regional_stats.len(), 1);
        assert!(!out.weekly_rewards.is_empty());
        let wk10 = out
            .weekly_rewards
            .iter()
            .find(|w| w.week_number == 10)
            .unwrap();
        for r in &wk10.per_farm_rewards {
            assert_eq!(r.amount, BigInt::from_u64(10000).unwrap());
        }
    }
}
