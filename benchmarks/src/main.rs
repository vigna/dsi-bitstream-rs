#![allow(unused_macros)]

use dsi_bitstream::prelude::*;
use rand::Rng;
use std::hint::black_box;

/// How many random codes we will write and read in the benchmark
const VALUES: usize = 1_000_000;
/// How many iterations to do before starting measuring, this is done to warmup
/// the caches and the branch predictor
const WARMUP_ITERS: usize = 100;
/// How many iterations of measurement we will execute
const BENCH_ITERS: usize = 11;
/// For how many times we will measure the measurement overhead
const CALIBRATION_ITERS: usize = 100_000;
/// To proprly test delta we compute a discrete version of the indended
/// distribution. The original distribution is infinite but we need to cut it
/// down to a finite set. This value represent the maximum value we are going to
/// extract
const DELTA_DISTR_SIZE: usize = 1_000_000;

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

#[cfg(feature = "rtdsc")]
mod rdtsc;
#[cfg(feature = "rtdsc")]
use rdtsc::*;

#[cfg(not(feature = "rtdsc"))]
use std::time::Instant;

mod metrics;
use metrics::*;

mod utils;
use utils::*;

mod data;
use data::*;

macro_rules! bench {
    ($cal:expr, $code:literal, $read:ident, $write:ident, $gen_data:ident, $bo:ident, $($table:expr),*) => {{
// the memory where we will write values
let mut buffer: Vec<WriteWord> = Vec::with_capacity(VALUES);
// counters for the total read time and total write time
#[cfg(feature="reads")]
let mut read_buff = MetricsStream::with_capacity(VALUES);
#[cfg(feature="reads")]
let mut read_unbuff = MetricsStream::with_capacity(VALUES);
#[cfg(not(feature="reads"))]
let mut write = MetricsStream::with_capacity(VALUES);

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
    write.avg / VALUES as f64,
    write.std / VALUES as f64,
    write.percentile_25 / VALUES as f64,
    write.median / VALUES as f64,
    write.percentile_75 / VALUES as f64,
);
#[cfg(feature="reads")]
println!("{}::{}::{},{},{},{},{},{},{},{}",
    $code, stringify!($bo), table, // the informations about what we are benchmarking
    "read_buff",
    ratio,
    read_buff.avg / VALUES as f64,
    read_buff.std / VALUES as f64,
    read_buff.percentile_25 / VALUES as f64,
    read_buff.median / VALUES as f64,
    read_buff.percentile_75 / VALUES as f64,
);
#[cfg(feature="reads")]
println!("{}::{}::{},{},{},{},{},{},{},{}",
    $code, stringify!($bo), table, // the informations about what we are benchmarking
    "read_unbuff",
    ratio,
    read_unbuff.avg / VALUES as f64,
    read_unbuff.std / VALUES as f64,
    read_unbuff.percentile_25 / VALUES as f64,
    read_unbuff.median / VALUES as f64,
    read_unbuff.percentile_75 / VALUES as f64,
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
    //unsafe{assert_ne!(libc::nice(-20-libc::nice(0)), -1);}

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
