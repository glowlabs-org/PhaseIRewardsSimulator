use crate::core_types::{Competition, CompetitionID};
use num_bigint::BigInt;
use num_traits::Zero;
use std::collections::HashMap;

fn scale_1e18() -> BigInt {
    BigInt::from(1_000_000_000_000_000_000u128)
}

fn default_region_weekly_glw(region: u64) -> BigInt {
    let base = match region {
        1 => BigInt::from(120_641u64), // cgp
        2 => BigInt::from(18_119u64),  // utah
        3 => BigInt::from(18_119u64),  // colorado
        4 => BigInt::from(18_119u64),  // missouri
        _ => BigInt::zero(),
    };
    base * scale_1e18()
}

pub fn apply_gctl_inflation(
    competitions: &mut HashMap<CompetitionID, Competition>,
    gctl_distribution: &Option<HashMap<u64, BigInt>>,
) {
    if competitions.is_empty() {
        return;
    }

    let get_weekly_glw: Box<dyn Fn(u64) -> BigInt> = if let Some(dist) = gctl_distribution {
        let mut total_gctl = BigInt::zero();
        for gctl_val in dist.values() {
            total_gctl += gctl_val;
        }

        if total_gctl.is_zero() {
            Box::new(|_| BigInt::zero())
        } else {
            let total_weekly_glw = BigInt::from(175_000u64) * scale_1e18();
            let dist_clone = dist.clone();
            let total_gctl_clone = total_gctl.clone();
            Box::new(move |region: u64| {
                let region_gctl = dist_clone
                    .get(&region)
                    .cloned()
                    .unwrap_or_else(BigInt::zero);
                (&total_weekly_glw * region_gctl) / &total_gctl_clone
            })
        }
    } else {
        Box::new(default_region_weekly_glw)
    };

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
        let mut by_region: HashMap<u64, Vec<(CompetitionID, BigInt)>> = HashMap::new();

        for (cid, comp) in competitions.iter() {
            if let Some(bucket) = comp.buckets.get(&week) {
                by_region
                    .entry(cid.region_id)
                    .or_default()
                    .push((cid.clone(), bucket.total_deposits.clone()));
            }
        }

        for (region, items) in by_region {
            let weekly_total = get_weekly_glw(region);
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
