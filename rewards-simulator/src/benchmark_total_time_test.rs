use crate::competition_simulator::simulate_with_diagnostics;
use crate::models::{InputData, SolarFarm};
use num_bigint::BigInt;
use std::time::Instant;

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

fn build_single_comp_input(farms_per_comp: usize, rng: &mut Rng) -> InputData {
    let mut farms = Vec::with_capacity(farms_per_comp);
    let region_id = "1".to_string();
    let asset_id = "glw".to_string();
    let mut addr_counter: u64 = 1;

    for j in 0..farms_per_comp {
        let farm_id = format!("{}-{}", region_id, j + 1);

        let weekly_cc = rng.range_inclusive(5, 500);
        let pd = rng.range_inclusive(1_000, 100_000);
        let ar = rng.range_inclusive(1_000, 100_000);
        let first_week = rng.range_inclusive(1, 100);
        let weeks_alive = rng.range_inclusive(80, 100);

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

    InputData {
        cgp_leftovers: Default::default(),
        solar_farms: farms,
    }
}

#[test]
#[ignore]
fn benchmark_total_time_single_comp_100k_farms() {
    let mut rng = Rng::new(0xC0FFEE_u64);
    let farms_per_comp: usize = 1_000;

    let input = build_single_comp_input(farms_per_comp, &mut rng);

    let t0 = Instant::now();
    let diag = simulate_with_diagnostics(input).expect("simulation should not error at API level");
    let dt = t0.elapsed();

    if !diag.errors.is_empty() {
        panic!(
            "consistency issues detected ({} errors), example: {}",
            diag.errors.len(),
            diag.errors
                .get(0)
                .cloned()
                .unwrap_or_else(|| "<no message>".to_string())
        );
    }

    println!(
        "total-time benchmark: comps=1 farms={} duration_secs={:.3}",
        farms_per_comp,
        dt.as_secs_f64()
    );
}
