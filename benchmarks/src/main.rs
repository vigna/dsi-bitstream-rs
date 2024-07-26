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
use std::borrow::Borrow;
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

#[cfg(not(feature = "reads"))] // not actually used, but needed for type system
type ReadWord = u64;

#[cfg(feature = "reads")]
type WriteWord = u64;

#[cfg(all(not(feature = "reads"), feature = "u16"))]
type WriteWord = u16;
#[cfg(all(not(feature = "reads"), feature = "u32"))]
type WriteWord = u32;
#[cfg(all(not(feature = "reads"), feature = "u64"))]
type WriteWord = u64;

use std::time::Instant;

pub mod metrics;
use metrics::*;

pub mod utils;
use utils::*;

pub mod data;
use data::*;

type ReaderUnbuff<'a, E> = BitReader<E, MemWordReader<WriteWord, &'a [WriteWord]>>;
type Reader<'a, E> = BufBitReader<E, MemWordReader<ReadWord, &'a [ReadWord]>>;
type Writer<'a, E> = BufBitWriter<E, MemWordWriterVec<WriteWord, &'a mut Vec<WriteWord>>>;

#[cfg(feature = "reads")]
trait ReaderUnbuffBounds<E: Endianness>: ReadCodes<E> {}
#[cfg(feature = "reads")]
impl<E: Endianness, B: ReadCodes<E>> ReaderUnbuffBounds<E> for B {}
#[cfg(not(feature = "reads"))]
trait ReaderUnbuffBounds<E> {}
#[cfg(not(feature = "reads"))]
impl<E: Endianness, B> ReaderUnbuffBounds<E> for B {}

fn bench<E: Endianness, S: Borrow<str>>(
    cal: u128,
    code_name: S,
    ratio: Option<f64>,
    data: &[u64],
    #[cfg(feature = "reads")] read_unbuff_fn: impl for<'a> Fn(&mut ReaderUnbuff<'a, E>) -> u64,
    #[cfg(feature = "reads")] read_fn: impl for<'a> Fn(&mut Reader<'a, E>) -> u64,
    write_fn: impl for<'a> Fn(&mut Writer<'a, E>, u64) -> usize,
) where
    for<'a> ReaderUnbuff<'a, E>: ReaderUnbuffBounds<E>,
    for<'a> Reader<'a, E>: ReadCodes<E>,
    for<'a> Writer<'a, E>: WriteCodes<E>,
{
    let mut features = String::new();
    #[cfg(feature = "u16")]
    features.push_str("u16 ");
    #[cfg(feature = "u32")]
    features.push_str("u32 ");
    #[cfg(feature = "u64")]
    features.push_str("u64 ");
    #[cfg(feature = "reads")]
    features.push_str("reads ");
    #[cfg(feature = "tables")]
    features.push_str("tables ");
    #[cfg(feature = "no_tables")]
    features.push_str("no_tables ");
    #[cfg(feature = "delta_gamma")]
    features.push_str("delta_gamma ");
    #[cfg(feature = "all")]
    features.push_str("all ");

    let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);

    #[cfg(feature = "reads")]
    let mut read_buff = MetricsStream::with_capacity(N);
    #[cfg(feature = "reads")]
    let mut read_unbuff = MetricsStream::with_capacity(N);
    #[cfg(not(feature = "reads"))]
    let mut write = MetricsStream::with_capacity(N);

    for iter in 0..(WARMUP_ITERS + BENCH_ITERS) {
        buffer.clear();

        // Writes (we need to do this also in the case of reads)
        {
            let mut r =
                BufBitWriter::<E, _>::new(MemWordWriterVec::<WriteWord, _>::new(&mut buffer));

            #[cfg(not(feature = "reads"))]
            let w_start = Instant::now();

            for value in data {
                black_box(write_fn(&mut r, *value));
            }

            #[cfg(not(feature = "reads"))]
            if iter >= WARMUP_ITERS {
                write.update((w_start.elapsed().as_nanos() - cal) as f64);
            }
        }

        // Buffered reads
        #[cfg(feature = "reads")]
        {
            let transmuted_buff: &[ReadWord] = unsafe {
                core::slice::from_raw_parts(
                    buffer.as_ptr() as *const ReadWord,
                    buffer.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
                )
            };

            let mut r =
                BufBitReader::<E, _>::new(MemWordReader::<ReadWord, _>::new(transmuted_buff));

            let r_start = Instant::now();
            for _ in data {
                black_box(read_fn(&mut r));
            }

            let nanos = r_start.elapsed().as_nanos();

            if iter >= WARMUP_ITERS {
                read_buff.update((nanos - cal) as f64);
            }
        }

        // Unbuffered reads
        #[cfg(feature = "reads")]
        {
            let mut r = BitReader::<E, _>::new(MemWordReader::new(buffer.as_slice()));

            let r_start = Instant::now();
            for _ in data {
                black_box(read_unbuff_fn(&mut r));
            }

            let nanos = r_start.elapsed().as_nanos();

            if iter >= WARMUP_ITERS {
                read_unbuff.update((nanos - cal) as f64);
            }
        }
    }

    #[cfg(feature = "reads")]
    let read_buff = read_buff.finalize();
    #[cfg(feature = "reads")]
    let read_unbuff = read_unbuff.finalize();
    #[cfg(not(feature = "reads"))]
    let write = write.finalize();

    let ratio = match ratio {
        Some(r) => r.to_string(),
        None => String::new(),
    };

    // print the results
    #[cfg(not(feature = "reads"))]
    println!(
        "{},{},{},{},write,{},{},{},{},{},{}",
        features,
        core::any::type_name::<WriteWord>()
            .split("::")
            .last()
            .unwrap(),
        core::any::type_name::<E>().split("::").last().unwrap(),
        code_name.borrow(), // the informations about what we are benchmarking
        ratio,
        write.avg / N as f64,
        write.std / N as f64,
        write.percentile_25 / N as f64,
        write.median / N as f64,
        write.percentile_75 / N as f64,
    );
    #[cfg(feature = "reads")]
    println!(
        "{},{},{},{},read_buff,{},{},{},{},{},{}",
        features,
        core::any::type_name::<ReadWord>()
            .split("::")
            .last()
            .unwrap(),
        core::any::type_name::<E>().split("::").last().unwrap(),
        code_name.borrow(), // the informations about what we are benchmarking
        ratio,
        read_buff.avg / N as f64,
        read_buff.std / N as f64,
        read_buff.percentile_25 / N as f64,
        read_buff.median / N as f64,
        read_buff.percentile_75 / N as f64,
    );
    #[cfg(feature = "reads")]
    println!(
        "{},{},{},{},read_unbuff,{},{},{},{},{},{}",
        features,
        core::any::type_name::<ReadWord>()
            .split("::")
            .last()
            .unwrap(),
        core::any::type_name::<E>().split("::").last().unwrap(),
        code_name.borrow(), // the informations about what we are benchmarking
        ratio,
        read_unbuff.avg / N as f64,
        read_unbuff.std / N as f64,
        read_unbuff.percentile_25 / N as f64,
        read_unbuff.median / N as f64,
        read_unbuff.percentile_75 / N as f64,
    );
}

pub fn main() {
    // tricks to reduce the noise
    #[cfg(target_os = "linux")]
    pin_to_core(5);

    // figure out how much overhead we add by measuring
    let calibration = calibrate_overhead();
    // print the header of the csv, unless we're running the delta_gamma test
    #[cfg(feature = "print_header")]
    println!("features,word,endianness,pat,type,ratio,ns_avg,ns_std,ns_perc25,ns_median,ns_perc75");

    // For delta we need to generate the data differently
    // because we have four cases, depending on whether
    // we use gamma tables or not.

    #[cfg(feature = "all")]
    {
        let unary_data = gen_unary_data();
        bench::<BE, _>(
            calibration,
            "unary",
            None,
            &unary_data,
            #[cfg(feature = "reads")]
            |r| r.read_unary().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_unary().unwrap(),
            |r, v| r.write_unary(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            "unary",
            None,
            &unary_data,
            #[cfg(feature = "reads")]
            |r| r.read_unary().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_unary().unwrap(),
            |r, v| r.write_unary(v).unwrap(),
        );

        for b in [2, 3, 4] {
            let golomb_data = gen_golomb_data(b);
            bench::<BE, _>(
                calibration,
                format!("golomb{}", subscript(b)),
                None,
                &golomb_data,
                #[cfg(feature = "reads")]
                |r| r.read_golomb(b).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_golomb(b).unwrap(),
                |r, v| r.write_golomb(v, b).unwrap(),
            );
            bench::<LE, _>(
                calibration,
                format!("golomb{}", subscript(b)),
                None,
                &golomb_data,
                #[cfg(feature = "reads")]
                |r| r.read_golomb(b).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_golomb(b).unwrap(),
                |r, v| r.write_golomb(v, b).unwrap(),
            );
        }

        for k in [2, 3] {
            let exp_golomb_data = gen_exp_golomb_data(k);
            bench::<BE, _>(
                calibration,
                format!("exp_golomb{}", subscript(k as u64)),
                None,
                &exp_golomb_data,
                #[cfg(feature = "reads")]
                |r| r.read_exp_golomb(k).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_exp_golomb(k).unwrap(),
                |r, v| r.write_exp_golomb(v, k).unwrap(),
            );
            bench::<LE, _>(
                calibration,
                format!("exp_golomb{}", subscript(k as u64)),
                None,
                &exp_golomb_data,
                #[cfg(feature = "reads")]
                |r| r.read_exp_golomb(k).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_exp_golomb(k).unwrap(),
                |r, v| r.write_exp_golomb(v, k).unwrap(),
            );
        }

        for log2_b in [2, 3] {
            let rice_data = gen_rice_data(log2_b);
            bench::<BE, _>(
                calibration,
                format!("rice{}", subscript(log2_b as u64)),
                None,
                &rice_data,
                #[cfg(feature = "reads")]
                |r| r.read_rice(log2_b).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_rice(log2_b).unwrap(),
                |r, v| r.write_rice(v, log2_b).unwrap(),
            );
            bench::<LE, _>(
                calibration,
                format!("rice{}", subscript(log2_b as u64)),
                None,
                &rice_data,
                #[cfg(feature = "reads")]
                |r| r.read_rice(log2_b).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_rice(log2_b).unwrap(),
                |r, v| r.write_rice(v, log2_b).unwrap(),
            );
        }

        for k in [2, 3] {
            let pi_data = gen_pi_data(k);
            bench::<BE, _>(
                calibration,
                format!("π{}", subscript(k)),
                None,
                &pi_data,
                #[cfg(feature = "reads")]
                |r| r.read_pi(k).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_pi(k).unwrap(),
                |r, v| r.write_pi(v, k).unwrap(),
            );
            bench::<LE, _>(
                calibration,
                format!("π{}", subscript(k)),
                None,
                &pi_data,
                #[cfg(feature = "reads")]
                |r| r.read_pi(k).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_pi(k).unwrap(),
                |r, v| r.write_pi(v, k).unwrap(),
            );
        }

        for k in [2, 3] {
            let pi_web_data = gen_pi_web_data(k);
            bench::<BE, _>(
                calibration,
                format!("π_web{}", subscript(k)),
                None,
                &pi_web_data,
                #[cfg(feature = "reads")]
                |r| r.read_pi_web(k).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_pi_web(k).unwrap(),
                |r, v| r.write_pi_web(v, k).unwrap(),
            );
            bench::<LE, _>(
                calibration,
                format!("π_web{}", subscript(k)),
                None,
                &pi_web_data,
                #[cfg(feature = "reads")]
                |r| r.read_pi_web(k).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_pi_web(k).unwrap(),
                |r, v| r.write_pi_web(v, k).unwrap(),
            );
        }

        for k in [2, 4] {
            let zeta_data = gen_zeta_data(k);
            bench::<BE, _>(
                calibration,
                format!("ζ{}", subscript(k)),
                None,
                &zeta_data,
                #[cfg(feature = "reads")]
                |r| r.read_zeta(k).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_zeta(k).unwrap(),
                |r, v| r.write_zeta(v, k).unwrap(),
            );
            bench::<LE, _>(
                calibration,
                format!("ζ{}", subscript(k)),
                None,
                &zeta_data,
                #[cfg(feature = "reads")]
                |r| r.read_zeta(k).unwrap(),
                #[cfg(feature = "reads")]
                |r| r.read_zeta(k).unwrap(),
                |r, v| r.write_zeta(v, k).unwrap(),
            );
        }
    }

    #[cfg(feature = "tables")]
    {
        let (gamma_ratio, gamma_data) = gen_gamma_data();
        bench::<BE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "γ::Table {} {} bits ",
                dsi_bitstream::codes::gamma_tables::TABLES_TYPE,
                dsi_bitstream::codes::gamma_tables::READ_BITS
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "γ::Table max {}",
                dsi_bitstream::codes::gamma_tables::WRITE_MAX
            ),
            Some(gamma_ratio),
            &gamma_data,
            #[cfg(feature = "reads")]
            |r| r.read_gamma_param::<true>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_gamma_param::<true>().unwrap(),
            |r, v| r.write_gamma_param::<true>(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "γ::Table {} {} bits ",
                dsi_bitstream::codes::gamma_tables::TABLES_TYPE,
                dsi_bitstream::codes::gamma_tables::READ_BITS
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "γ::Table max {}",
                dsi_bitstream::codes::gamma_tables::WRITE_MAX
            ),
            Some(gamma_ratio),
            &gamma_data,
            #[cfg(feature = "reads")]
            |r| r.read_gamma_param::<true>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_gamma_param::<true>().unwrap(),
            |r, v| r.write_gamma_param::<true>(v).unwrap(),
        );

        let (zeta3_ratio, zeta3_data) = gen_zeta3_data();
        bench::<BE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "ζ₃::Table {} {} bits",
                dsi_bitstream::codes::zeta_tables::TABLES_TYPE,
                dsi_bitstream::codes::zeta_tables::READ_BITS
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "ζ₃::Table max {}",
                dsi_bitstream::codes::zeta_tables::WRITE_MAX
            ),
            Some(zeta3_ratio),
            &zeta3_data,
            #[cfg(feature = "reads")]
            |r| r.read_zeta3_param::<true>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_zeta3_param::<true>().unwrap(),
            |r, v| r.write_zeta3_param::<true>(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "ζ₃::Table {} {} bits",
                dsi_bitstream::codes::zeta_tables::TABLES_TYPE,
                dsi_bitstream::codes::zeta_tables::READ_BITS
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "ζ₃::Table max {}",
                dsi_bitstream::codes::zeta_tables::WRITE_MAX
            ),
            Some(zeta3_ratio),
            &zeta3_data,
            #[cfg(feature = "reads")]
            |r| r.read_zeta3_param::<true>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_zeta3_param::<true>().unwrap(),
            |r, v| r.write_zeta3_param::<true>(v).unwrap(),
        );

        let (delta_ratio, delta_data) = gen_delta_data();
        bench::<BE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "δ::Table::Table {} {} bits",
                dsi_bitstream::codes::delta_tables::TABLES_TYPE,
                dsi_bitstream::codes::delta_tables::READ_BITS,
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "δ::Table max {}",
                dsi_bitstream::codes::delta_tables::WRITE_MAX,
            ),
            Some(delta_ratio),
            &delta_data,
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<true, false>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<true, false>().unwrap(),
            |r, v| r.write_delta_param::<true, false>(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "δ::Table::Table {} {} bits",
                dsi_bitstream::codes::delta_tables::TABLES_TYPE,
                dsi_bitstream::codes::delta_tables::READ_BITS,
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "δ::Table max {}",
                dsi_bitstream::codes::delta_tables::WRITE_MAX,
            ),
            Some(delta_ratio),
            &delta_data,
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<true, false>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<true, false>().unwrap(),
            |r, v| r.write_delta_param::<true, false>(v).unwrap(),
        );
    }

    #[cfg(feature = "no_tables")]
    {
        let (gamma_ratio, gamma_data) = gen_gamma_data();
        bench::<BE, _>(
            calibration,
            "γ",
            Some(gamma_ratio),
            &gamma_data,
            #[cfg(feature = "reads")]
            |r| r.read_gamma_param::<false>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_gamma_param::<false>().unwrap(),
            |r, v| r.write_gamma_param::<false>(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            "γ",
            Some(gamma_ratio),
            &gamma_data,
            #[cfg(feature = "reads")]
            |r| r.read_gamma_param::<false>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_gamma_param::<false>().unwrap(),
            |r, v| r.write_gamma_param::<false>(v).unwrap(),
        );

        let (zeta3_ratio, zeta3_data) = gen_zeta3_data();
        bench::<BE, _>(
            calibration,
            "ζ₃",
            Some(zeta3_ratio),
            &zeta3_data,
            #[cfg(feature = "reads")]
            |r| r.read_zeta3_param::<false>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_zeta3_param::<false>().unwrap(),
            |r, v| r.write_zeta3_param::<false>(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            "ζ₃",
            Some(zeta3_ratio),
            &zeta3_data,
            #[cfg(feature = "reads")]
            |r| r.read_zeta3_param::<false>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_zeta3_param::<false>().unwrap(),
            |r, v| r.write_zeta3_param::<false>(v).unwrap(),
        );

        let (delta_ratio, delta_data) = gen_delta_data();
        bench::<BE, _>(
            calibration,
            "δ",
            Some(delta_ratio),
            &delta_data,
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<false, false>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<false, false>().unwrap(),
            |r, v| r.write_delta_param::<false, false>(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            "δ",
            Some(delta_ratio),
            &delta_data,
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<false, false>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<false, false>().unwrap(),
            |r, v| r.write_delta_param::<false, false>(v).unwrap(),
        );
    }

    #[cfg(feature = "delta_gamma")]
    {
        let (delta_ratio, delta_data) = gen_delta_data();
        bench::<BE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "δ::Table {} {} bits γTable {} {} bits",
                dsi_bitstream::codes::delta_tables::TABLES_TYPE,
                dsi_bitstream::codes::delta_tables::READ_BITS,
                dsi_bitstream::codes::gamma_tables::TABLES_TYPE,
                dsi_bitstream::codes::gamma_tables::READ_BITS,
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "δ::Table max {} γTable max {}",
                dsi_bitstream::codes::delta_tables::WRITE_MAX,
                dsi_bitstream::codes::gamma_tables::WRITE_MAX,
            ),
            Some(delta_ratio),
            &delta_data,
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<true, true>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<true, true>().unwrap(),
            |r, v| r.write_delta_param::<true, true>(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "δ::Table {} {} bits γTable {} {} bits",
                dsi_bitstream::codes::delta_tables::TABLES_TYPE,
                dsi_bitstream::codes::delta_tables::READ_BITS,
                dsi_bitstream::codes::gamma_tables::TABLES_TYPE,
                dsi_bitstream::codes::gamma_tables::READ_BITS,
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "δ::Table max {} γTable max {}",
                dsi_bitstream::codes::delta_tables::WRITE_MAX,
                dsi_bitstream::codes::gamma_tables::WRITE_MAX,
            ),
            Some(delta_ratio),
            &delta_data,
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<true, true>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<true, true>().unwrap(),
            |r, v| r.write_delta_param::<true, true>(v).unwrap(),
        );
        bench::<BE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "δ::Table::γTable {} {} bits",
                dsi_bitstream::codes::gamma_tables::TABLES_TYPE,
                dsi_bitstream::codes::gamma_tables::READ_BITS,
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "δ::γTable max {}",
                dsi_bitstream::codes::gamma_tables::WRITE_MAX,
            ),
            Some(delta_ratio),
            &delta_data,
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<false, true>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<false, true>().unwrap(),
            |r, v| r.write_delta_param::<false, true>(v).unwrap(),
        );
        bench::<LE, _>(
            calibration,
            #[cfg(feature = "reads")]
            format!(
                "δ::Table::γTable {} {} bits",
                dsi_bitstream::codes::gamma_tables::TABLES_TYPE,
                dsi_bitstream::codes::gamma_tables::READ_BITS,
            ),
            #[cfg(not(feature = "reads"))]
            format!(
                "δ::γTable max {}",
                dsi_bitstream::codes::gamma_tables::WRITE_MAX,
            ),
            Some(delta_ratio),
            &delta_data,
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<false, true>().unwrap(),
            #[cfg(feature = "reads")]
            |r| r.read_delta_param::<false, true>().unwrap(),
            |r, v| r.write_delta_param::<false, true>(v).unwrap(),
        );
    }
}
