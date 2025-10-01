pub mod competition_simulator;
pub mod core_types;
pub mod errors;
pub mod gctl;
pub mod models;
pub mod preload;
pub mod serde_utils;
pub mod server;

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod advanced_five_farms_region_test;
#[cfg(test)]
mod advanced_five_farms_test;
#[cfg(test)]
mod benchmark_test;
#[cfg(test)]
mod benchmark_total_time_test;
#[cfg(test)]
mod competition_simulator_test;
#[cfg(test)]
mod competition_simulator_unit_test;
#[cfg(test)]
mod default_ui_farms_test;
#[cfg(test)]
mod gctl_test;
#[cfg(test)]
mod preload_v1_test;
#[cfg(test)]
mod scaling_test;
#[cfg(test)]
mod server_test;
#[cfg(test)]
mod weeks_alive_bounds_test;
