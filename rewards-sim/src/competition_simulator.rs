use crate::core_types::*;
use crate::errors::SimError;
use crate::models::*;
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

const WEEK_BOUND: u64 = 1 << 12;
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
    pub region_id: u64,
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
    pub asset_id: String,
    pub region_id: u64,
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
            region_id: farm.region_id,
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

        let reward_splits = farm.reward_split.clone();

        comp.farms.insert(
            farm.farm_id.clone(),
            FarmInfo {
                farm_id: farm.farm_id.clone(),
                protocol_deposit_value: farm.protocol_deposit_value.clone(),
                assets_required: farm.assets_required.clone(),
                first_week: farm.first_week,
                final_week: farm.first_week + farm.weeks_alive - 1,
                asset_id: farm.asset_id.clone(),
                region_id: farm.region_id,
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
            bucket.pool_net_assets = carry_assets;
            bucket.pool_net_deposits = carry_deposits;

            if bucket.total_impact_assets.is_zero() {
                return Err(SimError::algorithm(format!(
                    "bucket at week {week} has zero total impact assets"
                )));
            }

            let farm_order = farm_iter_order(bucket);

            let mut stage1_results: Vec<(String, FarmBucketState, BigInt)> =
                Vec::with_capacity(farm_order.len());
            for fid in &farm_order {
                let mut state = bucket
                    .farm_states
                    .get(fid)
                    .cloned()
                    .ok_or_else(|| SimError::internal("missing state"))?;

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

            for (fid, mut state, deposits_recovered) in stage1_results {
                let fmeta = comp
                    .farms
                    .get(&fid)
                    .ok_or_else(|| SimError::internal("missing farm meta"))?;

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

                if cid.region_id == 1 && cid.asset_id.to_lowercase() == "usdg" {
                    if let Some(leftover) = input.cgp_leftovers.get(&week) {
                        if !bucket.total_deposits.is_zero() {
                            let bonus = (&deposits_recovered * leftover) / &bucket.total_deposits;
                            rewards += bonus;
                        }
                    }
                }

                state.rewards_this_week = rewards;
                bucket.farm_states.insert(fid.clone(), state.clone());

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
                let tol_ast = if cid.asset_id.to_lowercase() == "usdg" {
                    tol_dollars()
                } else {
                    tol_assets()
                };

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

    crate::gctl::apply_gctl_inflation(&mut competitions, &input.gctl_distribution);

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
                            region_id: finfo.region_id,
                            amount: st.rewards_this_week.clone(),
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

    let competitions_detailed = build_detailed_competitions(&competitions);

    Ok(SimulationDiagnostics {
        output,
        errors: diagnostics,
        competitions: competitions_detailed,
    })
}

pub fn simulate_multi_asset(
    mut input: InputDataMultiAsset,
    preload_v1: bool,
) -> Result<(BTreeMap<String, PublicWeekOutputMultiAsset>, Vec<String>), SimError> {
    let mut v1_farms: Vec<SolarFarm> = Vec::new();
    if preload_v1 {
        let v1_input = crate::preload::load_v1_data()?;
        for (week, amount) in v1_input.cgp_leftovers {
            let entry = input.cgp_leftovers.entry(week).or_default();
            *entry += amount;
        }
        v1_farms = v1_input.solar_farms;
    }

    let mut expanded_farms = Vec::new();
    let mut virtual_map: HashMap<String, (String, String)> = HashMap::new();
    let mut original_farm_map: HashMap<String, MultiAssetSolarFarm> = HashMap::new();

    for farm in input.solar_farms {
        if farm.assets.is_empty() {
            return Err(SimError::validation(format!(
                "Farm {} has no assets",
                farm.farm_id
            )));
        }

        for asset in &farm.assets {
            let aid = asset.asset_id.to_uppercase();
            if aid != "USDG" && aid != "GLW" && aid != "SGCTL" {
                return Err(SimError::validation(format!(
                    "Invalid assetId '{}' in farm {}. Must be USDG, GLW, or SGCTL",
                    asset.asset_id, farm.farm_id
                )));
            }

            if let Some(d) = asset.decimals {
                if aid == "GLW" && d != 18 {
                    return Err(SimError::validation(format!(
                        "Invalid decimals for GLW in farm {}: expected 18, got {}",
                        farm.farm_id, d
                    )));
                }
                if (aid == "USDG" || aid == "SGCTL") && d != 6 {
                    return Err(SimError::validation(format!(
                        "Invalid decimals for {} in farm {}: expected 6, got {}",
                        aid, farm.farm_id, d
                    )));
                }
            }
        }

        original_farm_map.insert(farm.farm_id.clone(), farm.clone());

        if farm.total_protocol_deposit_value.is_zero() {
            return Err(SimError::validation(format!(
                "Farm {} has zero totalProtocolDepositValue",
                farm.farm_id
            )));
        }

        for asset in &farm.assets {
            let virtual_id = format!("{}|{}", farm.farm_id, asset.asset_id);
            if virtual_map.contains_key(&virtual_id) {
                return Err(SimError::validation(format!(
                    "Collision or duplicate asset in farm: {}",
                    virtual_id
                )));
            }
            virtual_map.insert(
                virtual_id.clone(),
                (farm.farm_id.clone(), asset.asset_id.clone()),
            );

            let impact = (&farm.net_weekly_impact_assets * &asset.assets_required_usdc)
                / &farm.total_protocol_deposit_value;

            expanded_farms.push(SolarFarm {
                farm_id: virtual_id,
                asset_id: asset.asset_id.clone(),
                region_id: farm.region_id,
                weekly_impact_assets: impact,
                protocol_deposit_value: asset.assets_required_usdc.clone(),
                assets_required: asset.assets_required.clone(),
                reward_split: farm.reward_split.clone(),
                first_week: farm.first_week,
                weeks_alive: farm.weeks_alive,
            });
        }
    }

    for v1_farm in v1_farms {
        if original_farm_map.contains_key(&v1_farm.farm_id) {
            return Err(SimError::validation(format!(
                "Duplicate farm ID with V1 data: {}",
                v1_farm.farm_id
            )));
        }
        let ma_farm = MultiAssetSolarFarm {
            farm_id: v1_farm.farm_id.clone(),
            region_id: v1_farm.region_id,
            net_weekly_impact_assets: v1_farm.weekly_impact_assets.clone(),
            total_protocol_deposit_value: v1_farm.protocol_deposit_value.clone(),
            assets: vec![AssetRequirement {
                asset_id: v1_farm.asset_id.clone(),
                assets_required: v1_farm.assets_required.clone(),
                assets_required_usdc: v1_farm.protocol_deposit_value.clone(),
                quoted_by_gve_price_per_asset: BigInt::zero(),
                decimals: None,
            }],
            reward_split: v1_farm.reward_split.clone(),
            first_week: v1_farm.first_week,
            weeks_alive: v1_farm.weeks_alive,
        };
        original_farm_map.insert(v1_farm.farm_id.clone(), ma_farm);
        virtual_map.insert(
            v1_farm.farm_id.clone(),
            (v1_farm.farm_id.clone(), v1_farm.asset_id.clone()),
        );
        expanded_farms.push(v1_farm);
    }

    let single_asset_input = InputData {
        cgp_leftovers: input.cgp_leftovers,
        solar_farms: expanded_farms,
        gctl_distribution: input.gctl_distribution,
        output_farms: None,
    };

    let diag = simulate_with_diagnostics(single_asset_input)?;

    let mut result = build_multi_asset_output(
        &diag.competitions,
        &virtual_map,
        &original_farm_map,
        &diag.errors,
    );

    if let Some(output_farms) = input.output_farms {
        let allowed: HashSet<String> = output_farms.into_iter().collect();
        for week_out in result.values_mut() {
            week_out
                .farm_rewards
                .retain(|fr| allowed.contains(&fr.farm_id));
            week_out.wallet_distributions.clear();
            week_out.region_data.clear();
        }
    }

    Ok((result, diag.errors))
}

fn build_multi_asset_output(
    competitions: &[DetailedCompetition],
    virtual_map: &HashMap<String, (String, String)>,
    original_farm_map: &HashMap<String, MultiAssetSolarFarm>,
    warnings: &[String],
) -> BTreeMap<String, PublicWeekOutputMultiAsset> {
    struct FarmAgg {
        assets_earned: BTreeMap<String, BigInt>,
        glow_inflation: BigInt,
        expected_production: BigInt, // per sub-farm part
    }
    struct Agg {
        farm_aggs: HashMap<String, FarmAgg>,
        region_data: BTreeMap<u64, BTreeMap<String, RegionAssetSummary>>,
        wallets: BTreeMap<String, WalletDistributionMultiAsset>,
    }
    let mut by_week: BTreeMap<u64, Agg> = BTreeMap::new();

    for comp in competitions {
        for b in &comp.buckets {
            let entry = by_week.entry(b.week_number).or_insert_with(|| Agg {
                farm_aggs: HashMap::new(),
                region_data: BTreeMap::new(),
                wallets: BTreeMap::new(),
            });

            // Region Data
            let rkey = comp.region_id;
            let reg_map = entry.region_data.entry(rkey).or_default();
            let sums = reg_map
                .entry(comp.asset_id.clone())
                .or_insert(RegionAssetSummary {
                    protocol_deposit_sum: BigInt::zero(),
                    carbon_credit_production_sum: BigInt::zero(),
                });
            sums.protocol_deposit_sum += b.total_deposits.clone();
            sums.carbon_credit_production_sum += b.total_impact_assets.clone();

            for st in &b.farm_states {
                // st.farm_id is Virtual ID
                let Some((orig_id, asset_id)) = virtual_map.get(&st.farm_id) else {
                    continue;
                };
                let Some(orig_farm) = original_farm_map.get(orig_id) else {
                    continue;
                };

                // Farm aggregation
                let f_agg = entry
                    .farm_aggs
                    .entry(orig_id.clone())
                    .or_insert_with(|| FarmAgg {
                        assets_earned: BTreeMap::new(),
                        glow_inflation: BigInt::zero(),
                        expected_production: BigInt::zero(),
                    });

                *f_agg.assets_earned.entry(asset_id.clone()).or_default() += &st.rewards_this_week;
                f_agg.expected_production += &st.impact_assets_contributed;

                let glw_per_virtual = if b.total_deposits.is_zero() {
                    BigInt::zero()
                } else {
                    (&b.glw_inflation * &st.deposits_contributed) / &b.total_deposits
                };
                f_agg.glow_inflation += &glw_per_virtual;

                // Wallet Distributions
                let million = BigInt::from(1_000_000u32);
                for sp in &orig_farm.reward_split {
                    let asset_part =
                        (&st.rewards_this_week * &sp.deposit_split_percent_6_decimals) / &million;
                    let glw_part =
                        (&glw_per_virtual * &sp.glow_split_percent_6_decimals) / &million;

                    let addr_key = sp.wallet_address.clone();
                    let wallet = entry.wallets.entry(addr_key.clone()).or_insert_with(|| {
                        WalletDistributionMultiAsset {
                            user_address: addr_key.clone(),
                            assets_earned: BTreeMap::new(),
                            glow_inflation_earned: BigInt::zero(),
                            traces: Vec::new(),
                        }
                    });

                    let ae = wallet
                        .assets_earned
                        .entry(asset_id.clone())
                        .or_insert_with(|| "0".to_string());
                    let cur = ae.parse::<BigInt>().unwrap_or_else(|_| BigInt::zero());
                    let new = cur + asset_part.clone();
                    *ae = new.to_string();

                    wallet.glow_inflation_earned += glw_part.clone();

                    wallet.traces.push(WalletTraceMultiAsset {
                        farm_id: orig_id.clone(),
                        asset_id: asset_id.clone(),
                        amount: asset_part,
                        region_id: comp.region_id,
                        inflation_reward_split_6_decimals: sp.glow_split_percent_6_decimals.clone(),
                        deposit_reward_split_6_decimals: sp
                            .deposit_split_percent_6_decimals
                            .clone(),
                        glow_inflation_reward: glw_part,
                    });
                }
            }
        }
    }

    let mut out: BTreeMap<String, PublicWeekOutputMultiAsset> = BTreeMap::new();
    for (w, agg) in by_week {
        let mut wd: Vec<WalletDistributionMultiAsset> = agg.wallets.into_values().collect();
        wd.sort_by(|a, b| a.user_address.cmp(&b.user_address));

        let mut fr_list = Vec::new();
        for (orig_id, f_agg) in agg.farm_aggs {
            let Some(orig_farm) = original_farm_map.get(&orig_id) else {
                continue;
            };
            let mut assets_out = Vec::new();
            for (aid, amt) in f_agg.assets_earned {
                assets_out.push(AssetRewardOut {
                    asset_id: aid,
                    asset_earned: amt,
                });
            }
            assets_out.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));

            fr_list.push(FarmRewardMultiAsset {
                id: format!("{}-week-{}", orig_id, w),
                farm_id: orig_id.clone(),
                week_index: w,
                region_id: orig_farm.region_id,
                assets: assets_out,
                glow_inflation_reward: f_agg.glow_inflation,
                protocol_deposit: orig_farm.total_protocol_deposit_value.clone(),
                expected_production: orig_farm.net_weekly_impact_assets.clone(), // Use authoritative from farm, not sum of parts
            });
        }
        fr_list.sort_by(|a, b| a.id.cmp(&b.id));

        out.insert(
            w.to_string(),
            PublicWeekOutputMultiAsset {
                wallet_distributions: wd,
                farm_rewards: fr_list,
                region_data: agg.region_data,
                warnings: warnings.to_vec(),
            },
        );
    }
    out
}

fn build_detailed_competitions(
    competitions: &HashMap<CompetitionID, Competition>,
) -> Vec<DetailedCompetition> {
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
        let mut farms_vec: Vec<DetailedFarmInfo> = comp
            .farms
            .values()
            .map(|f| DetailedFarmInfo {
                farm_id: f.farm_id.clone(),
                protocol_deposit_value: f.protocol_deposit_value.clone(),
                assets_required: f.assets_required.clone(),
                first_week: f.first_week,
                final_week: f.final_week,
                asset_id: f.asset_id.clone(),
                region_id: f.region_id,
                reward_splits: f.reward_splits.clone(),
            })
            .collect();
        farms_vec.sort_by(|a, b| a.farm_id.cmp(&b.farm_id));

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
            region_id: cid.region_id,
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
        if f.asset_id.trim().is_empty() {
            return Err(SimError::validation("empty asset_id"));
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

        if f.reward_split.is_empty() {
            return Err(SimError::validation(format!(
                "farm {} must have a non-empty rewardSplit",
                f.farm_id
            )));
        }

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
    pub region_id: u64,
    #[serde(serialize_with = "crate::serde_utils::bigint_to_string")]
    pub glow_inflation_reward: BigInt,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletDistribution {
    pub user_address: String,
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
    pub region_id: u64,
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
pub struct PublicWeekOutput {
    pub wallet_distributions: Vec<WalletDistribution>,
    pub farm_rewards: Vec<FarmRewardOut>,
    pub region_data: BTreeMap<u64, BTreeMap<String, RegionAssetSummary>>,
    pub warnings: Vec<String>,
}

pub fn perform_consistency_checks(
    week: u64,
    wd: &[WalletDistribution],
    fr: &[FarmRewardOut],
    warnings: &mut Vec<String>,
) {
    // Per-wallet trace consistency checks
    for wallet_dist in wd {
        let traces_glow_sum: BigInt = wallet_dist
            .traces
            .iter()
            .map(|t| &t.glow_inflation_reward)
            .sum();
        if (&wallet_dist.glow_inflation_earned - &traces_glow_sum).abs() > tol_assets() {
            warnings.push(format!(
                    "Consistency Warning (Week {week}): Wallet {} glow inflation mismatch. Earned: {}, Traces Sum: {}",
                    wallet_dist.user_address, wallet_dist.glow_inflation_earned, traces_glow_sum
                ));
        }

        let mut traces_assets: BTreeMap<String, BigInt> = BTreeMap::new();
        for trace in &wallet_dist.traces {
            *traces_assets.entry(trace.asset.clone()).or_default() += &trace.amount;
        }
        let mut earned_assets: BTreeMap<String, BigInt> = BTreeMap::new();
        for (asset, amount_str) in &wallet_dist.assets_earned {
            earned_assets.insert(
                asset.clone(),
                amount_str.parse().unwrap_or_else(|_| BigInt::zero()),
            );
        }

        let all_assets: HashSet<_> = earned_assets.keys().chain(traces_assets.keys()).collect();
        for asset in all_assets {
            let earned = earned_assets.get(asset).cloned().unwrap_or_default();
            let traces_sum = traces_assets.get(asset).cloned().unwrap_or_default();
            let tolerance = if asset.to_lowercase() == "usdg" {
                tol_dollars()
            } else {
                tol_assets()
            };
            if (&earned - &traces_sum).abs() > tolerance {
                warnings.push(format!(
                        "Consistency Warning (Week {week}): Wallet {} asset '{asset}' amount mismatch. Earned: {earned}, Traces Sum: {traces_sum}",
                        wallet_dist.user_address
                    ));
            }
        }
    }

    // Week-total consistency checks
    let total_glow_wallets: BigInt = wd.iter().map(|w| &w.glow_inflation_earned).sum();
    let total_glow_farms: BigInt = fr.iter().map(|f| &f.glow_inflation_reward).sum();
    if (&total_glow_wallets - &total_glow_farms).abs() > tol_assets() {
        warnings.push(format!(
            "Consistency Warning (Week {week}): Total glow inflation mismatch. Wallets sum: {total_glow_wallets}, Farms sum: {total_glow_farms}"
        ));
    }

    let mut total_assets_wallets: BTreeMap<String, BigInt> = BTreeMap::new();
    for wallet in wd {
        for (asset, amount_str) in &wallet.assets_earned {
            let amount: BigInt = amount_str.parse().unwrap_or_else(|_| BigInt::zero());
            *total_assets_wallets.entry(asset.clone()).or_default() += amount;
        }
    }
    let mut total_assets_farms: BTreeMap<String, BigInt> = BTreeMap::new();
    for farm in fr {
        *total_assets_farms.entry(farm.asset.clone()).or_default() += &farm.asset_earned;
    }

    let all_total_assets: HashSet<_> = total_assets_wallets
        .keys()
        .chain(total_assets_farms.keys())
        .collect();
    for asset in all_total_assets {
        let wallets_total = total_assets_wallets.get(asset).cloned().unwrap_or_default();
        let farms_total = total_assets_farms.get(asset).cloned().unwrap_or_default();
        let tolerance = if asset.to_lowercase() == "usdg" {
            tol_dollars()
        } else {
            tol_assets()
        };
        if (&wallets_total - &farms_total).abs() > tolerance {
            warnings.push(format!(
                "Consistency Warning (Week {week}): Total asset '{asset}' mismatch. Wallets sum: {wallets_total}, Farms sum: {farms_total}"
            ));
        }
    }
}

pub fn build_public_output_from_detailed(
    competitions: &[DetailedCompetition],
    warnings: &[String],
) -> BTreeMap<String, PublicWeekOutput> {
    struct Agg {
        farm_rewards: Vec<FarmRewardOut>,
        region_data: BTreeMap<u64, BTreeMap<String, RegionAssetSummary>>,
        wallets: BTreeMap<String, WalletDistribution>,
    }
    let mut by_week: BTreeMap<u64, Agg> = BTreeMap::new();

    for comp in competitions {
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

            let rkey = comp.region_id;
            let reg_map = entry.region_data.entry(rkey).or_default();
            let sums = reg_map
                .entry(comp.asset_id.clone())
                .or_insert(RegionAssetSummary {
                    protocol_deposit_sum: BigInt::zero(),
                    carbon_credit_production_sum: BigInt::zero(),
                });
            sums.protocol_deposit_sum += b.total_deposits.clone();
            sums.carbon_credit_production_sum += b.total_impact_assets.clone();

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
                    region_id: finfo.region_id,
                    asset_earned: st.rewards_this_week.clone(),
                    glow_inflation_reward: glw_per_farm.clone(),
                    protocol_deposit: finfo.protocol_deposit_value.clone(),
                    expected_production: st.impact_assets_contributed.clone(),
                });

                let million = BigInt::from(1_000_000u32);
                for sp in &finfo.reward_splits {
                    let asset_part =
                        (&st.rewards_this_week * &sp.deposit_split_percent_6_decimals) / &million;
                    let glw_part = (&glw_per_farm * &sp.glow_split_percent_6_decimals) / &million;

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
                        inflation_reward_split_6_decimals: sp.glow_split_percent_6_decimals.clone(),
                        deposit_reward_split_6_decimals: sp
                            .deposit_split_percent_6_decimals
                            .clone(),
                        amount: asset_part,
                        region_id: finfo.region_id,
                        glow_inflation_reward: glw_part,
                    });
                }
            }
        }
    }

    let mut out: BTreeMap<String, PublicWeekOutput> = BTreeMap::new();
    for (w, agg) in by_week {
        let mut wd: Vec<WalletDistribution> = agg.wallets.into_values().collect();
        let fr = agg.farm_rewards;
        let mut current_warnings = warnings.to_vec();

        perform_consistency_checks(w, &wd, &fr, &mut current_warnings);

        wd.sort_by(|a, b| a.user_address.cmp(&b.user_address));
        let mut sorted_fr = fr;
        sorted_fr.sort_by(|a, b| a.id.cmp(&b.id));

        out.insert(
            w.to_string(),
            PublicWeekOutput {
                wallet_distributions: wd,
                farm_rewards: sorted_fr,
                region_data: agg.region_data,
                warnings: current_warnings,
            },
        );
    }
    out
}
