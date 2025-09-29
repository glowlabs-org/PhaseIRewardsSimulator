use crate::error::PreprocessorError;
use crate::v1_format::{V1History, V1RewardSplit};
use crate::v2_format::{V2Configuration, V2RewardSplit, V2SolarFarm};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

struct InternalV2SolarFarm {
    farm_id: String,
    asset_id: String,
    region_id: String,
    net_weekly_carbon_credits: BigInt,
    protocol_deposit_value: BigInt,
    assets_required: BigInt,
    first_week: u64,
    weeks_alive: u64,
    reward_splits: Vec<V2RewardSplit>,
}

impl From<InternalV2SolarFarm> for V2SolarFarm {
    fn from(internal_farm: InternalV2SolarFarm) -> Self {
        V2SolarFarm {
            farm_id: internal_farm.farm_id,
            asset_id: internal_farm.asset_id,
            region_id: internal_farm.region_id,
            net_weekly_carbon_credits: internal_farm.net_weekly_carbon_credits.to_string(),
            protocol_deposit_value: internal_farm.protocol_deposit_value.to_string(),
            assets_required: internal_farm.assets_required.to_string(),
            first_week: internal_farm.first_week,
            weeks_alive: internal_farm.weeks_alive,
            reward_splits: internal_farm.reward_splits,
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
    let mut farm_ids = HashSet::new();
    for v1_farm in history.solar_farms {
        if !farm_ids.insert(v1_farm.farm_id.clone()) {
            return Err(PreprocessorError::InvalidInput(format!(
                "Duplicate farmId: {}",
                v1_farm.farm_id
            )));
        }
        validate_reward_splits(&v1_farm.reward_splits)?;

        let weeks_alive =
            1 + ((208.0 - 96.0 + v1_farm.first_rewards_week as f64) / 2.08).floor() as u64;
        let net_weekly_carbon_credits = (v1_farm.net_weekly_carbon_credits * 1_000_000.0).round();
        let nwcc_bigint = BigInt::from(net_weekly_carbon_credits as i64);

        let v2_farm = InternalV2SolarFarm {
            farm_id: v1_farm.farm_id.clone(),
            asset_id: "usdg".to_string(),
            region_id: "cgp".to_string(),
            net_weekly_carbon_credits: nwcc_bigint,
            protocol_deposit_value: BigInt::from(0),
            assets_required: BigInt::from(0),
            first_week: 96,
            weeks_alive,
            reward_splits: v1_farm
                .reward_splits
                .into_iter()
                .map(|rs| V2RewardSplit {
                    wallet_address: rs.wallet_address,
                    glow_split_percent_6_decimals: rs.glow_split_percent_6_decimals,
                    deposit_split_percent_6_decimals: rs.deposit_split_percent_6_decimals,
                })
                .collect(),
        };
        v2_farms.insert(v1_farm.farm_id, v2_farm);
    }

    for deposit in history.protocol_deposits {
        if let Some(farm) = v2_farms.get_mut(&deposit.corresponding_farm) {
            let usdg_provided = BigInt::from_str(&deposit.usdg_provided)?;
            farm.protocol_deposit_value += &usdg_provided;
            farm.assets_required += &usdg_provided;

            let usdg_provided_f64 = usdg_provided.to_f64().ok_or_else(|| {
                PreprocessorError::InvalidInput(format!(
                    "Cannot convert usdgProvided to f64: {usdg_provided}"
                ))
            })?;
            let deduction = BigInt::from((usdg_provided_f64 / 192.0).ceil() as i64);

            for i in (deposit.week_provided + 16)..(deposit.week_provided + 208) {
                if i < 96 {
                    continue;
                }
                if let Some(leftover) = cgp_leftovers.get_mut(&i) {
                    *leftover -= &deduction;
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

    let final_cgp_leftovers = cgp_leftovers
        .into_iter()
        .map(|(week, amount)| (week.to_string(), amount.to_string()))
        .collect();

    let mut final_solar_farms: Vec<V2SolarFarm> =
        v2_farms.into_values().map(V2SolarFarm::from).collect();
    final_solar_farms.sort_by(|a, b| a.farm_id.cmp(&b.farm_id));

    Ok(V2Configuration {
        cgp_leftovers: final_cgp_leftovers,
        solar_farms: final_solar_farms,
    })
}

fn validate_reward_splits(splits: &[V1RewardSplit]) -> Result<(), PreprocessorError> {
    let mut glow_sum = BigInt::from(0);
    let mut deposit_sum = BigInt::from(0);

    for split in splits {
        glow_sum += BigInt::from_str(&split.glow_split_percent_6_decimals)?;
        deposit_sum += BigInt::from_str(&split.deposit_split_percent_6_decimals)?;
    }

    let expected = BigInt::from(1_000_000);
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
