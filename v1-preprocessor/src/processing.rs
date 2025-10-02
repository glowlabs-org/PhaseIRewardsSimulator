use crate::error::PreprocessorError;
use crate::v1_format::{V1History, V1RewardSplit};
use crate::v2_format::{V2Configuration, V2RewardSplit, V2SolarFarm};
use num_bigint::BigInt;
use num_traits::Signed;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

struct InternalV2SolarFarm {
    farm_id: String,
    asset_id: String,
    region_id: String,
    net_weekly_impact_assets: BigInt,
    protocol_deposit_value: BigInt,
    assets_required: BigInt,
    first_week: u64,
    weeks_alive: u64,
    reward_split: Vec<V2RewardSplit>,
}

impl From<InternalV2SolarFarm> for V2SolarFarm {
    fn from(internal_farm: InternalV2SolarFarm) -> Self {
        V2SolarFarm {
            farm_id: internal_farm.farm_id,
            asset_id: internal_farm.asset_id,
            region_id: internal_farm.region_id,
            net_weekly_impact_assets: internal_farm.net_weekly_impact_assets.to_string(),
            protocol_deposit_value: internal_farm.protocol_deposit_value.to_string(),
            assets_required: internal_farm.assets_required.to_string(),
            first_week: internal_farm.first_week,
            weeks_alive: internal_farm.weeks_alive,
            reward_split: internal_farm.reward_split,
        }
    }
}

pub fn process_v1_history(history: V1History) -> Result<V2Configuration, PreprocessorError> {
    let mut cgp_leftovers: HashMap<u64, BigInt> = history
        .usdg_per_week
        .into_iter()
        .map(|(week_str, amount_str)| {
            let week = week_str.parse::<u64>().map_err(|_| {
                PreprocessorError::InvalidInput(format!("Invalid week number: {week_str}"))
            })?;
            let amount = BigInt::from_str(&amount_str)?;
            Ok((week, amount))
        })
        .collect::<Result<HashMap<_, _>, PreprocessorError>>()?;

    let mut v2_farms: HashMap<String, InternalV2SolarFarm> = HashMap::new();

    for (farm_id, v1_farm) in history.solar_farms {
        validate_reward_split(&v1_farm.reward_split)?;

        let weeks_alive =
            1 + ((208.0 - 97.0 + v1_farm.first_reward_week as f64) / 2.08).floor() as u64;

        let scaled = (v1_farm.net_weekly_impact_assets * 1e18f64).round() as i128;
        let nwia_bigint = BigInt::from(scaled);

        let v2_farm = InternalV2SolarFarm {
            farm_id: farm_id.clone(),
            asset_id: "USDG".to_string(),
            region_id: "cgp".to_string(),
            net_weekly_impact_assets: nwia_bigint,
            protocol_deposit_value: BigInt::from(0u32),
            assets_required: BigInt::from(0u32),
            first_week: 97,
            weeks_alive,
            reward_split: v1_farm
                .reward_split
                .into_iter()
                .map(|rs| V2RewardSplit {
                    wallet_address: rs.wallet_address,
                    glow_split_percent_6_decimals: rs.glow_split_percent_6_decimals,
                    deposit_split_percent_6_decimals: rs.deposit_split_percent_6_decimals,
                })
                .collect(),
        };
        if v2_farms.insert(farm_id.clone(), v2_farm).is_some() {
            return Err(PreprocessorError::InvalidInput(format!(
                "Duplicate farmId encountered in input: {farm_id}"
            )));
        }
    }

    for deposit in history.protocol_deposits {
        if let Some(farm) = v2_farms.get_mut(&deposit.corresponding_farm) {
            let usdg_provided = BigInt::from_str(&deposit.usdg_provided)?;
            farm.protocol_deposit_value += &usdg_provided;
            farm.assets_required += &usdg_provided;

            // ceil(usdg_provided / 192)
            let deduction = (&usdg_provided + BigInt::from(191u32)) / BigInt::from(192u32);

            let start = deposit.week_provided + 16;
            let end_exclusive = deposit.week_provided + 208;

            for i in start..end_exclusive {
                if i < 97 {
                    continue;
                }
                let leftover = cgp_leftovers.get_mut(&i).ok_or_else(|| {
                    PreprocessorError::InvalidInput(format!(
                        "cgpLeftovers missing entry for week {i} while applying protocol deposit \
                         for farm '{}' (weekProvided: {})",
                        deposit.corresponding_farm, deposit.week_provided
                    ))
                })?;
                *leftover -= &deduction;

                if *leftover < BigInt::from(-10i32) {
                    return Err(PreprocessorError::InvalidInput(format!(
                        "cgpLeftovers for week {i} fell below -10 ({leftover}) while applying protocol deposit \
                         for farm '{}' (weekProvided: {}, usdgProvided: {}). This exceeds allowed dust.",
                        deposit.corresponding_farm, deposit.week_provided, deposit.usdg_provided
                    )));
                }
            }
        }
    }

    for migration in history.migrating_to_utah {
        if let Some(farm) = v2_farms.get_mut(&migration.farm_id) {
            farm.region_id = "utah".to_string();
            farm.protocol_deposit_value =
                BigInt::from_str(&migration.updated_protocol_deposit_value)?;
        } else {
            return Err(PreprocessorError::InvalidInput(format!(
                "Farm to migrate not found: {}",
                migration.farm_id
            )));
        }
    }

    // Build final cgpLeftovers:
    // - Remove any entries with week < 97
    // - If any value < -10, error
    // - If value is negative but >= -10, prune (do not include in output)
    let mut final_cgp_leftovers: BTreeMap<u64, String> = BTreeMap::new();
    for (week, amount) in cgp_leftovers.into_iter() {
        if week < 97 {
            continue;
        }
        if amount < BigInt::from(-10i32) {
            return Err(PreprocessorError::InvalidInput(format!(
                "cgpLeftovers for week {week} is below -10 after processing ({amount})."
            )));
        }
        if amount.is_negative() {
            continue;
        }
        final_cgp_leftovers.insert(week, amount.to_string());
    }

    let mut final_solar_farms: Vec<V2SolarFarm> =
        v2_farms.into_values().map(V2SolarFarm::from).collect();
    final_solar_farms.sort_by(|a, b| {
        a.weeks_alive
            .cmp(&b.weeks_alive)
            .then_with(|| a.farm_id.cmp(&b.farm_id))
    });

    Ok(V2Configuration {
        cgp_leftovers: final_cgp_leftovers,
        solar_farms: final_solar_farms,
    })
}

fn validate_reward_split(splits: &[V1RewardSplit]) -> Result<(), PreprocessorError> {
    let mut glow_sum = BigInt::from(0u32);
    let mut deposit_sum = BigInt::from(0u32);

    for split in splits {
        glow_sum += BigInt::from_str(&split.glow_split_percent_6_decimals)?;
        deposit_sum += BigInt::from_str(&split.deposit_split_percent_6_decimals)?;
    }

    let expected = BigInt::from(1_000_000u32);
    if glow_sum != expected {
        return Err(PreprocessorError::InvalidInput(format!(
            "glowSplitPercent6Decimals do not sum to 1000000, got {glow_sum}"
        )));
    }
    if deposit_sum != expected {
        return Err(PreprocessorError::InvalidInput(format!(
            "depositSplitPercent6Decimals do not sum to 1000000, got {deposit_sum}"
        )));
    }

    Ok(())
}
