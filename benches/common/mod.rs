#[cfg(not(feature = "implied"))]
compile_error!("Benchmarks require the `implied` feature: use --features implied");

pub mod data;
pub mod utils;

/// Number of read/write operations tested for each combination of parameters.
pub const N: usize = 1_000_000;
