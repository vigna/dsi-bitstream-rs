/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![doc = include_str!("../README.md")]
#![allow(unused_macros)]

use dsi_bitstream::prelude::*;
use rand::Rng;
use std::hint::black_box;

/// Number of read/write operations tested for each combination of parameters.
pub const N: usize = 1_000_000;
/// Number of warmup read/write operations.
pub const WARMUP_ITERS: usize = 3;
/// How many iterations of measurement we will execute.
pub const BENCH_ITERS: usize = 11;
/// For how many times we will measure the measurement overhead.
pub const CALIBRATION_ITERS: usize = 100_000;

#[cfg(all(feature = "reads", feature = "u16"))]
type ReadWord = u16;
#[cfg(all(feature = "reads", feature = "u32"))]
type ReadWord = u32;
#[cfg(all(feature = "reads", feature = "u64"))]
type ReadWord = u64;

#[cfg(feature = "reads")]
type WriteWord = u64;

#[cfg(all(not(feature = "reads"), feature = "u16"))]
type WriteWord = u16;
#[cfg(all(not(feature = "reads"), feature = "u32"))]
type WriteWord = u32;
#[cfg(all(not(feature = "reads"), feature = "u64"))]
type WriteWord = u64;

#[cfg(not(feature = "rtdsc"))]
use std::time::Instant;

pub mod metrics;
use metrics::*;

pub mod utils;
use utils::*;

pub mod data;
use data::*;

#[doc(hidden)]
macro_rules! bench {
    ($cal:expr, $code:literal, $read:ident, $write:ident, $gen_data:ident, $bo:ident, $($table:expr),*) => {{

let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);

#[cfg(feature="reads")]
let mut read_buff = MetricsStream::with_capacity(N);
#[cfg(feature="reads")]
let mut read_unbuff = MetricsStream::with_capacity(N);
#[cfg(not(feature="reads"))]
let mut write = MetricsStream::with_capacity(N);

let (ratio, data) = $gen_data();

for iter in 0..(WARMUP_ITERS + BENCH_ITERS) {
    buffer.clear();

    // Writes (we need to do this also in the case of reads)
    {
        let mut r = BufBitWriter::<$bo, _>::new(
            MemWordWriterVec::<WriteWord, _>::new(&mut buffer)
        );

        #[cfg(not(feature="reads"))]
        let w_start = Instant::now();

        for value in &data {
            black_box(r.$write::<$($table),*>(*value).unwrap());
        }

        #[cfg(not(feature="reads"))]
        if iter >= WARMUP_ITERS {
            write.update((w_start.elapsed().as_nanos() - $cal) as f64);
        }
    }

    // Buffered reads
    #[cfg(feature="reads")]
    {
        let transmuted_buff: &[ReadWord] = unsafe{core::slice::from_raw_parts(
            buffer.as_ptr() as *const ReadWord,
            buffer.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
        )};

        let mut r = BufBitReader::<$bo, _>::new(
            MemWordReader::<ReadWord, _>::new(&transmuted_buff)
        );

        let r_start = Instant::now();
        for _ in &data {
            black_box(r.$read::<$($table),*>().unwrap());
        }

        let nanos = r_start.elapsed().as_nanos();

        if iter >= WARMUP_ITERS {
            read_buff.update((nanos - $cal) as f64);
        }
    }

    // Unbuffered reads
    #[cfg(feature="reads")]
    {
        let mut r = BitReader::<$bo, _>::new(
            MemWordReader::new(&buffer)
        );

        let r_start = Instant::now();
        for _ in &data {
            black_box(r.$read::<$($table),*>().unwrap());
        }

        let nanos = r_start.elapsed().as_nanos();

        if iter >= WARMUP_ITERS {
            read_unbuff.update((nanos - $cal) as f64);
        }
    }
}

#[cfg(feature="reads")]
let read_buff = read_buff.finalize();
#[cfg(feature="reads")]
let read_unbuff = read_unbuff.finalize();
#[cfg(not(feature="reads"))]
let write = write.finalize();

let table = if ($($table),*,).0 {
    "Table"
} else {
    "NoTable"
};
// print the results
#[cfg(not(feature="reads"))]
println!("{}::{}::{},{},{},{},{},{},{},{}",
    $code, stringify!($bo), table, // the informations about what we are benchmarking
    "write",
    ratio,
    write.avg / N as f64,
    write.std / N as f64,
    write.percentile_25 / N as f64,
    write.median / N as f64,
    write.percentile_75 / N as f64,
);
#[cfg(feature="reads")]
println!("{}::{}::{},{},{},{},{},{},{},{}",
    $code, stringify!($bo), table, // the informations about what we are benchmarking
    "read_buff",
    ratio,
    read_buff.avg / N as f64,
    read_buff.std / N as f64,
    read_buff.percentile_25 / N as f64,
    read_buff.median / N as f64,
    read_buff.percentile_75 / N as f64,
);
#[cfg(feature="reads")]
println!("{}::{}::{},{},{},{},{},{},{},{}",
    $code, stringify!($bo), table, // the informations about what we are benchmarking
    "read_unbuff",
    ratio,
    read_unbuff.avg / N as f64,
    read_unbuff.std / N as f64,
    read_unbuff.percentile_25 / N as f64,
    read_unbuff.median / N as f64,
    read_unbuff.percentile_75 / N as f64,
);

}};
}

#[doc(hidden)]
macro_rules! impl_code {
    ($cal:expr, $code:literal, $read:ident, $write:ident, $gen_data:ident) => {
        bench!($cal, $code, $read, $write, $gen_data, BE, false);
        bench!($cal, $code, $read, $write, $gen_data, BE, true);
        bench!($cal, $code, $read, $write, $gen_data, LE, false);
        bench!($cal, $code, $read, $write, $gen_data, LE, true);
    };
}

pub fn main() {
    // tricks to reduce the noise
    #[cfg(target_os = "linux")]
    pin_to_core(5);

    // figure out how much overhead we add by measuring
    let calibration = calibrate_overhead();
    // print the header of the csv, unless we're running the delta_gamma test
    #[cfg(not(feature = "delta_gamma"))]
    println!("pat,type,ratio,ns_avg,ns_std,ns_perc25,ns_median,ns_perc75");

    // For delta we need to generate the data differently
    // because we have four cases, depending on whether
    // we use gamma tables or not.

    #[cfg(not(feature = "delta_gamma"))]
    {
        impl_code!(
            calibration,
            "gamma",
            read_gamma_param,
            write_gamma_param,
            gen_gamma_data
        );
        impl_code!(
            calibration,
            "zeta3",
            read_zeta3_param,
            write_zeta3_param,
            gen_zeta3_data
        );
        // delta with gamma tables disabled
        bench!(
            calibration,
            "delta",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            BE,
            true,  // use δ tables
            false  // use γ tables
        );
        bench!(
            calibration,
            "delta",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            BE,
            false, // use δ tables
            false  // use γ tables
        );
        bench!(
            calibration,
            "delta",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            LE,
            true,  // use δ tables
            false  // use γ tables
        );
        bench!(
            calibration,
            "delta",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            LE,
            false, // use δ tables
            false  // use γ tables
        );
    }

    #[cfg(feature = "delta_gamma")]
    {
        // delta with gamma tables enabled
        bench!(
            calibration,
            "delta_gamma",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            BE,
            true, // use δ tables
            true  // use γ tables
        );
        bench!(
            calibration,
            "delta_gamma",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            BE,
            false, // use δ tables
            true   // use γ tables
        );
        bench!(
            calibration,
            "delta_gamma",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            LE,
            true, // use δ tables
            true  // use γ tables
        );
        bench!(
            calibration,
            "delta_gamma",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            LE,
            false, // use δ tables
            true   // use γ tables
        );
    }
}
