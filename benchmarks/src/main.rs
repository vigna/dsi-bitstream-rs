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
/// How many iterations of measurement we will execute
pub const BENCH_ITERS: usize = 11;
/// For how many times we will measure the measurement overhead
pub const CALIBRATION_ITERS: usize = 100_000;
/// To proprly test delta we compute a discrete version of the indended
/// distribution. The original distribution is infinite but we need to cut it
/// down to a finite set. This value represent the maximum value we are going to
/// extract
pub const DELTA_DISTR_SIZE: usize = 1_000_000;

// Conditional compilation requires to set a feature for the word size
// ("u16", "u32", or "u64") and the feature "reads" to test reads
// instead of writes.

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

macro_rules! bench {
    ($cal:expr, $code:literal, $read:ident, $write:ident, $gen_data:ident, $bo:ident, $($table:expr),*) => {{
// the memory where we will write values
let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
// counters for the total read time and total write time
#[cfg(feature="reads")]
let mut read_buff = MetricsStream::with_capacity(N);
#[cfg(feature="reads")]
let mut read_unbuff = MetricsStream::with_capacity(N);
#[cfg(not(feature="reads"))]
let mut write = MetricsStream::with_capacity(N);

// measure
let (ratio, data) = $gen_data();

for iter in 0..(WARMUP_ITERS + BENCH_ITERS) {
    buffer.clear();
    // write the codes
    {
        // init the writer
        let mut r = BufBitWriter::<$bo, _>::new(
            MemWordWriterVec::<WriteWord, _>::new(&mut buffer)
        );
        // measure
        #[cfg(not(feature="reads"))]
        let w_start = Instant::now();
        for value in &data {
            black_box(r.$write::<$($table),*>(*value).unwrap());
        }
        // add the measurement if we are not in the warmup
        #[cfg(not(feature="reads"))]
        if iter >= WARMUP_ITERS {
            write.update((w_start.elapsed().as_nanos() - $cal) as f64);
        }
    }

    #[cfg(feature="reads")]
    // read the codes
    {
        let transmuted_buff: &[ReadWord] = unsafe{core::slice::from_raw_parts(
            buffer.as_ptr() as *const ReadWord,
            buffer.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
        )};

        // init the reader
        let mut r = BufBitReader::<$bo, _, dsi_bitstream::prelude::table_params::DefaultReadParams>::new(
            MemWordReader::<ReadWord, _>::new(&transmuted_buff)
        );
        // measure
        let r_start = Instant::now();
        for _ in &data {
            black_box(r.$read::<$($table),*>().unwrap());
        }
        let nanos =  r_start.elapsed().as_nanos();
        // add the measurement if we are not in the warmup
        if iter >= WARMUP_ITERS {
            read_buff.update((nanos - $cal) as f64);
        }
    }

    #[cfg(feature="reads")]
    {
        // init the reader
        let mut r = BitReader::<$bo, _>::new(
            MemWordReader::new(&buffer)
        );
        // measure
        let r_start = Instant::now();
        for _ in &data {
            black_box(r.$read::<$($table),*>().unwrap());
        }
        let nanos =  r_start.elapsed().as_nanos();
        // add the measurement if we are not in the warmup
        if iter >= WARMUP_ITERS {
            read_unbuff.update((nanos - $cal) as f64);
        }
    }
}

// convert from cycles to nano seconds
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

/// macro to implement all combinations of bit order and table use
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
    // print the header of the csv
    #[cfg(not(feature = "delta_gamma"))]
    println!("pat,type,ratio,ns_avg,ns_std,ns_perc25,ns_median,ns_perc75");

    // For delta we need to generate the data differently
    // because we have four cases, depending on whether
    // we use gamma tables or not.

    #[cfg(not(feature = "delta_gamma"))]
    {
        impl_code!(
            calibration,
            "unary",
            read_unary_param,
            write_unary_param,
            gen_unary_data
        );
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
            true,
            false
        );
        bench!(
            calibration,
            "delta",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            BE,
            false,
            false
        );
        bench!(
            calibration,
            "delta",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            LE,
            true,
            false
        );
        bench!(
            calibration,
            "delta",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            LE,
            false,
            false
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
            true,
            true
        );
        bench!(
            calibration,
            "delta_gamma",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            BE,
            false,
            true
        );
        bench!(
            calibration,
            "delta_gamma",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            LE,
            true,
            true
        );
        bench!(
            calibration,
            "delta_gamma",
            read_delta_param,
            write_delta_param,
            gen_delta_data,
            LE,
            false,
            true
        );
    }
}
