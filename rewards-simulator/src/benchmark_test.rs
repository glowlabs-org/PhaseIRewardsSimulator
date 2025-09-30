use crate::models::{InputData, SolarFarm};
use crate::simulator::simulate_with_diagnostics;
use num_bigint::BigInt;
use std::time::{Duration, Instant};

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn range_inclusive(&mut self, low: u64, high: u64) -> u64 {
        if low >= high {
            return low;
        }
        let span = high - low + 1;
        (self.next_u64() % span) + low
    }
}

fn random_address(i: u64) -> String {
    let mut bytes = [0u8; 20];
    let mut v = i.wrapping_mul(0x9E3779B97F4A7C15);
    for b in &mut bytes {
        v ^= v >> 12;
        v ^= v << 25;
        v ^= v >> 27;
        *b = (v & 0xFF) as u8;
        v = v.wrapping_mul(0x2545F4914F6CDD1D);
    }
    let mut s = String::from("0x");
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn build_input(num_comps: usize, farms_per_comp: usize, rng: &mut Rng) -> InputData {
    let mut farms = Vec::with_capacity(num_comps * farms_per_comp);
    let mut addr_counter: u64 = 1;

    for c in 0..num_comps {
        let region_id = (c + 1).to_string();
        let asset_id = "glw".to_string();

        for j in 0..farms_per_comp {
            let farm_id = format!("{}-{}", region_id, j + 1);

            let weekly_cc = rng.range_inclusive(5, 500);
            let pd = rng.range_inclusive(1_000, 100_000);
            let ar = rng.range_inclusive(1_000, 100_000);
            let first_week = rng.range_inclusive(1, 100);
            let weeks_alive = rng.range_inclusive(2, 100);

            farms.push(SolarFarm {
                farm_id,
                asset_id: asset_id.clone(),
                region_id: region_id.clone(),
                weekly_carbon_credits: BigInt::from(weekly_cc),
                protocol_deposit_value: BigInt::from(pd),
                assets_required: BigInt::from(ar),
                rewards_address: random_address(addr_counter),
                first_week,
                weeks_alive,
            });
            addr_counter = addr_counter.wrapping_add(1);
        }
    }

    InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    }
}

#[test]
#[ignore]
fn scaling_benchmark_iterative() {
    let mut rng = Rng::new(0xBADC0FFEE_u64);
    let mut num_comps: usize = 5;
    let mut farms_per_comp: usize = 10;

    let per_iter_threshold = Duration::from_secs(1);
    let total_budget = Duration::from_secs(60);
    let mut total_elapsed = Duration::from_secs(0);

    let mut iterations_run = 0usize;

    loop {
        let input = build_input(num_comps, farms_per_comp, &mut rng);
        let t0 = Instant::now();
        let diag = match simulate_with_diagnostics(input) {
            Ok(d) => d,
            Err(e) => panic!("simulation failed unexpectedly: {e}"),
        };
        let dt = t0.elapsed();
        total_elapsed += dt;
        iterations_run += 1;

        println!(
            "benchmark iteration {}: comps={} farms_per_comp={} total_farms={} duration_secs={:.3}",
            iterations_run,
            num_comps,
            farms_per_comp,
            num_comps * farms_per_comp,
            dt.as_secs_f64()
        );

        if !diag.errors.is_empty() {
            panic!(
                "consistency issues detected in iteration {} ({} errors), example: {}",
                iterations_run,
                diag.errors.len(),
                diag.errors
                    .get(0)
                    .cloned()
                    .unwrap_or_else(|| "<no message>".to_string())
            );
        }

        if dt >= per_iter_threshold {
            break;
        }
        if total_elapsed >= total_budget {
            break;
        }
        num_comps += 5;
        farms_per_comp = farms_per_comp.saturating_mul(2);
    }

    assert!(
        iterations_run >= 1,
        "benchmark should run at least one iteration"
    );
}
