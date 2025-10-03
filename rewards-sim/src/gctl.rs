use crate::core_types::{Competition, CompetitionID};
use num_bigint::BigInt;
use num_traits::Zero;
use std::collections::HashMap;

fn scale_1e18() -> BigInt {
    BigInt::from(1_000_000_000_000_000_000u128)
}

fn region_weekly_glw(region: &str) -> BigInt {
    let base = match region {
        "cgp" => BigInt::from(120_641u64),
        "utah" => BigInt::from(18_119u64),
        "colorado" => BigInt::from(18_119u64),
        "missouri" => BigInt::from(18_119u64),
        _ => BigInt::zero(),
    };
    base * scale_1e18()
}

pub fn apply_gctl_inflation(competitions: &mut HashMap<CompetitionID, Competition>) {
    if competitions.is_empty() {
        return;
    }

    let mut global_first = u64::MAX;
    let mut global_last = 0u64;
    for comp in competitions.values() {
        global_first = global_first.min(comp.first_week);
        global_last = global_last.max(comp.final_week);
    }
    if global_first == u64::MAX {
        return;
    }

    for week in global_first..=global_last {
        let mut by_region: HashMap<String, Vec<(CompetitionID, BigInt)>> = HashMap::new();

        for (cid, comp) in competitions.iter() {
            if let Some(bucket) = comp.buckets.get(&week) {
                by_region
                    .entry(cid.region_id.clone())
                    .or_default()
                    .push((cid.clone(), bucket.total_deposits.clone()));
            }
        }

        for (region, items) in by_region {
            let weekly_total = region_weekly_glw(&region);
            if weekly_total.is_zero() {
                for (cid, _) in items {
                    if let Some(b) = competitions
                        .get_mut(&cid)
                        .and_then(|c| c.buckets.get_mut(&week))
                    {
                        b.glw_inflation = BigInt::zero();
                    }
                }
                continue;
            }

            let mut sum_deposits = BigInt::zero();
            for (_, dep) in &items {
                sum_deposits += dep;
            }

            if sum_deposits.is_zero() {
                for (cid, _) in items {
                    if let Some(b) = competitions
                        .get_mut(&cid)
                        .and_then(|c| c.buckets.get_mut(&week))
                    {
                        b.glw_inflation = BigInt::zero();
                    }
                }
                continue;
            }

            for (cid, dep) in items {
                let share = (&weekly_total * dep) / &sum_deposits;
                if let Some(b) = competitions
                    .get_mut(&cid)
                    .and_then(|c| c.buckets.get_mut(&week))
                {
                    b.glw_inflation = share;
                }
            }
        }
    }
}
