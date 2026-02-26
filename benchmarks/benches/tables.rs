/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Table-sweep Criterion benchmarks for dsi-bitstream codes.
//!
//! Tests each code with current table configuration, varying table sizes.
//! The Python scripts (`bench_code_tables_read.py`, `bench_code_tables_write.py`)
//! call this binary repeatedly with different table sizes.

use benchmarks::data::*;
use benchmarks::N;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use dsi_bitstream::prelude::*;
use std::hint::black_box;

#[cfg(feature = "u16")]
type ReadWord = u16;
#[cfg(feature = "u32")]
type ReadWord = u32;
#[cfg(feature = "u64")]
type ReadWord = u64;

#[cfg(feature = "reads")]
type WriteWord = u64;

#[cfg(all(not(feature = "reads"), feature = "u16"))]
type WriteWord = u16;
#[cfg(all(not(feature = "reads"), feature = "u32"))]
type WriteWord = u32;
#[cfg(all(not(feature = "reads"), feature = "u64"))]
type WriteWord = u64;

/// Macro to register table-sweep benchmarks for a single code.
/// Generates separate closures for BufBitReader and BitReader (unbuffered)
/// since they are different types.
/// Data (`$data`) and hit ratio (`$ratio`) are passed in to avoid
/// regenerating them for every table/no-table variant of the same code.
macro_rules! bench_code_tables {
    (
        $group:expr, $code_name:literal,
        $read_fn:ident, $write_fn:ident, $ratio:expr, $data:expr,
        $($table_param:expr),*
    ) => {{
        let table_str = if ($($table_param),*,).0 { "Table" } else { "NoTable" };
        let ratio = $ratio;
        let data = $data;

        // Print hit ratio to stderr for the Python scripts to capture
        eprintln!("RATIO:{}::BE::{},{:.6}", $code_name, table_str, ratio);
        eprintln!("RATIO:{}::LE::{},{:.6}", $code_name, table_str, ratio);

        #[cfg(not(feature = "reads"))]
        {
            // Write benchmark — BE
            {
                let bench_id = format!("{}::BE::{}/write", $code_name, table_str);
                let data_ref = data;
                $group.bench_function(&bench_id, |b| {
                    b.iter(|| {
                        let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                        let mut w = BufBitWriter::<BE, _>::new(
                            MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                        );
                        for &value in data_ref {
                            black_box(w.$write_fn::<$($table_param),*>(value).unwrap());
                        }
                    });
                });
            }
            // Write benchmark — LE
            {
                let bench_id = format!("{}::LE::{}/write", $code_name, table_str);
                let data_ref = data;
                $group.bench_function(&bench_id, |b| {
                    b.iter(|| {
                        let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                        let mut w = BufBitWriter::<LE, _>::new(
                            MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                        );
                        for &value in data_ref {
                            black_box(w.$write_fn::<$($table_param),*>(value).unwrap());
                        }
                    });
                });
            }
        }

        #[cfg(feature = "reads")]
        {
            // Encode data for reads
            let encoded_be = {
                let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                {
                    let mut w = BufBitWriter::<BE, _>::new(
                        MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                    );
                    for &value in data {
                        w.$write_fn::<$($table_param),*>(value).unwrap();
                    }
                }
                buffer
            };
            let encoded_le = {
                let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                {
                    let mut w = BufBitWriter::<LE, _>::new(
                        MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                    );
                    for &value in data {
                        w.$write_fn::<$($table_param),*>(value).unwrap();
                    }
                }
                buffer
            };

            let n = data.len();

            // Buffered read — BE
            {
                let bench_id = format!("{}::BE::{}/read_b", $code_name, table_str);
                $group.bench_function(&bench_id, |b| {
                    b.iter(|| {
                        // SAFETY: Vec<u64> is aligned to 8 bytes, which satisfies
                        // alignment for ReadWord (u16/u32/u64).
                        let slice: &[ReadWord] = unsafe { encoded_be.align_to::<ReadWord>().1 };
                        let mut r = BufBitReader::<BE, _>::new(
                            MemWordReader::<ReadWord, _>::new(slice),
                        );
                        for _ in 0..n {
                            black_box(r.$read_fn::<$($table_param),*>().unwrap());
                        }
                    });
                });
            }
            // Buffered read — LE
            {
                let bench_id = format!("{}::LE::{}/read_b", $code_name, table_str);
                $group.bench_function(&bench_id, |b| {
                    b.iter(|| {
                        // SAFETY: Vec<u64> is aligned to 8 bytes, which satisfies
                        // alignment for ReadWord (u16/u32/u64).
                        let slice: &[ReadWord] = unsafe { encoded_le.align_to::<ReadWord>().1 };
                        let mut r = BufBitReader::<LE, _>::new(
                            MemWordReader::<ReadWord, _>::new(slice),
                        );
                        for _ in 0..n {
                            black_box(r.$read_fn::<$($table_param),*>().unwrap());
                        }
                    });
                });
            }
            // Unbuffered read — BE
            {
                let bench_id = format!("{}::BE::{}/read_ub", $code_name, table_str);
                $group.bench_function(&bench_id, |b| {
                    b.iter(|| {
                        let mut r = BitReader::<BE, _>::new(
                            MemWordReader::new(&encoded_be),
                        );
                        for _ in 0..n {
                            black_box(r.$read_fn::<$($table_param),*>().unwrap());
                        }
                    });
                });
            }
            // Unbuffered read — LE
            {
                let bench_id = format!("{}::LE::{}/read_ub", $code_name, table_str);
                $group.bench_function(&bench_id, |b| {
                    b.iter(|| {
                        let mut r = BitReader::<LE, _>::new(
                            MemWordReader::new(&encoded_le),
                        );
                        for _ in 0..n {
                            black_box(r.$read_fn::<$($table_param),*>().unwrap());
                        }
                    });
                });
            }
        }
    }};
}

/// Table-sweep benchmarks: tests codes with current table configuration.
/// The Python scripts call this repeatedly with different table sizes.
fn bench_tables(c: &mut Criterion) {
    #[cfg(target_os = "linux")]
    benchmarks::utils::pin_to_core(5);

    let mut group = c.benchmark_group("tables");
    group.throughput(Throughput::Elements(N as u64));

    let univ = cfg!(feature = "univ");

    #[cfg(not(feature = "delta_gamma"))]
    {
        let (ratio, data) = gen_gamma_data(univ);
        bench_code_tables!(group, "gamma", read_gamma_param, write_gamma_param, ratio, &data, true);
        bench_code_tables!(group, "gamma", read_gamma_param, write_gamma_param, ratio, &data, false);

        let (ratio, data) = gen_zeta3_data(univ);
        bench_code_tables!(group, "zeta3", read_zeta3_param, write_zeta3_param, ratio, &data, true);
        bench_code_tables!(group, "zeta3", read_zeta3_param, write_zeta3_param, ratio, &data, false);

        let (ratio, data) = gen_pi2_data(univ);
        bench_code_tables!(group, "pi2", read_pi2_param, write_pi2_param, ratio, &data, true);
        bench_code_tables!(group, "pi2", read_pi2_param, write_pi2_param, ratio, &data, false);

        let (ratio, data) = gen_omega_data(univ);
        bench_code_tables!(group, "omega", read_omega_param, write_omega_param, ratio, &data, true);
        bench_code_tables!(group, "omega", read_omega_param, write_omega_param, ratio, &data, false);

        // delta without gamma tables
        let (ratio, data) = gen_delta_data(univ);
        bench_code_tables!(group, "delta", read_delta_param, write_delta_param, ratio, &data, true, false);
        bench_code_tables!(group, "delta", read_delta_param, write_delta_param, ratio, &data, false, false);
    }

    #[cfg(feature = "delta_gamma")]
    {
        // delta with gamma tables
        let (ratio, data) = gen_delta_data(univ);
        bench_code_tables!(group, "delta_g", read_delta_param, write_delta_param, ratio, &data, true, true);
        bench_code_tables!(group, "delta_g", read_delta_param, write_delta_param, ratio, &data, false, true);
    }

    group.finish();
}

criterion_group! {
    name = tables;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_secs(1))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = bench_tables
}

criterion_main!(tables);
