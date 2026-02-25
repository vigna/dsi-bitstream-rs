/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Comparative Criterion benchmarks for dsi-bitstream codes.
//!
//! Compares all codes side by side using both implied and universal distributions.
//!
//! Environment variables for filtering:
//!   BENCH_CODES=gamma,delta    (default: all)
//!   BENCH_ENDIAN=BE            (default: both)
//!   BENCH_DIST=implied         (default: both)
//!   BENCH_OPS=read,write     (default: all)

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

/// Checks if a given value is in the comma-separated env var, or returns true
/// if the env var is not set (meaning "all").
fn env_filter(var: &str, value: &str) -> bool {
    match std::env::var(var) {
        Ok(v) => v.split(',').any(|s| s.trim() == value),
        Err(_) => true,
    }
}

/// Macro to register a comparative benchmark for a code (both endiannesses,
/// both distributions, read + write).
macro_rules! bench_comp {
    ($group:expr, $name:literal, $write_method:ident, $read_method:ident, $len_fn:expr) => {
        if env_filter("BENCH_CODES", $name) {
            let dists: Vec<(&str, bool)> = {
                let mut v = Vec::new();
                if env_filter("BENCH_DIST", "implied") {
                    v.push(("implied", false));
                }
                if env_filter("BENCH_DIST", "univ") {
                    v.push(("univ", true));
                }
                v
            };

            for &(dist_name, univ) in &dists {
                let data = gen_data($len_fn, univ);

                // Write benchmarks
                if env_filter("BENCH_OPS", "write") {
                    if env_filter("BENCH_ENDIAN", "BE") {
                        let bench_id = format!("{}/BE/{}/write", $name, dist_name);
                        let data_ref = &data;
                        $group.bench_function(&bench_id, |b| {
                            b.iter(|| {
                                let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                                let mut w = BufBitWriter::<BE, _>::new(
                                    MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                                );
                                for &value in data_ref {
                                    black_box(w.$write_method(value).unwrap());
                                }
                            });
                        });
                    }
                    if env_filter("BENCH_ENDIAN", "LE") {
                        let bench_id = format!("{}/LE/{}/write", $name, dist_name);
                        let data_ref = &data;
                        $group.bench_function(&bench_id, |b| {
                            b.iter(|| {
                                let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                                let mut w = BufBitWriter::<LE, _>::new(
                                    MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                                );
                                for &value in data_ref {
                                    black_box(w.$write_method(value).unwrap());
                                }
                            });
                        });
                    }
                }

                // Read benchmarks
                if env_filter("BENCH_OPS", "read") {
                    if env_filter("BENCH_ENDIAN", "BE") {
                        let encoded = {
                            let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                            {
                                let mut w = BufBitWriter::<BE, _>::new(
                                    MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                                );
                                for &value in &data {
                                    w.$write_method(value).unwrap();
                                }
                            }
                            buffer
                        };
                        let n = data.len();
                        let bench_id = format!("{}/BE/{}/read", $name, dist_name);
                        $group.bench_function(&bench_id, |b| {
                            b.iter(|| {
                                // SAFETY: Vec<u64> is aligned to 8 bytes, which
                                // satisfies alignment for ReadWord (u16/u32/u64).
                                let slice: &[ReadWord] =
                                    unsafe { encoded.align_to::<ReadWord>().1 };
                                let mut r = BufBitReader::<BE, _>::new(
                                    MemWordReader::<ReadWord, _>::new(slice),
                                );
                                for _ in 0..n {
                                    black_box(r.$read_method().unwrap());
                                }
                            });
                        });
                    }
                    if env_filter("BENCH_ENDIAN", "LE") {
                        let encoded = {
                            let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                            {
                                let mut w = BufBitWriter::<LE, _>::new(
                                    MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                                );
                                for &value in &data {
                                    w.$write_method(value).unwrap();
                                }
                            }
                            buffer
                        };
                        let n = data.len();
                        let bench_id = format!("{}/LE/{}/read", $name, dist_name);
                        $group.bench_function(&bench_id, |b| {
                            b.iter(|| {
                                // SAFETY: Vec<u64> is aligned to 8 bytes, which
                                // satisfies alignment for ReadWord (u16/u32/u64).
                                let slice: &[ReadWord] =
                                    unsafe { encoded.align_to::<ReadWord>().1 };
                                let mut r = BufBitReader::<LE, _>::new(
                                    MemWordReader::<ReadWord, _>::new(slice),
                                );
                                for _ in 0..n {
                                    black_box(r.$read_method().unwrap());
                                }
                            });
                        });
                    }
                }
            }
        }
    };
}

/// Macro variant for parametric codes that take a `k` parameter.
/// Uses method name + k directly in the closures to avoid lifetime issues.
macro_rules! bench_comp_k {
    ($group:expr, $name:expr, $write_method:ident($k:expr), $read_method:ident($rk:expr), $len_fn:expr) => {
        {
            let name_str: String = $name;
            let k = $k;
            let rk = $rk;
            if env_filter("BENCH_CODES", &name_str) {
                let dists: Vec<(&str, bool)> = {
                    let mut v = Vec::new();
                    if env_filter("BENCH_DIST", "implied") {
                        v.push(("implied", false));
                    }
                    if env_filter("BENCH_DIST", "univ") {
                        v.push(("univ", true));
                    }
                    v
                };

                for &(dist_name, univ) in &dists {
                    let data = gen_data($len_fn, univ);

                    if env_filter("BENCH_OPS", "write") {
                        if env_filter("BENCH_ENDIAN", "BE") {
                            let bench_id = format!("{}/BE/{}/write", name_str, dist_name);
                            let data_ref = &data;
                            $group.bench_function(&bench_id, |b| {
                                b.iter(|| {
                                    let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                                    let mut w = BufBitWriter::<BE, _>::new(
                                        MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                                    );
                                    for &value in data_ref {
                                        black_box(w.$write_method(value, k).unwrap());
                                    }
                                });
                            });
                        }
                        if env_filter("BENCH_ENDIAN", "LE") {
                            let bench_id = format!("{}/LE/{}/write", name_str, dist_name);
                            let data_ref = &data;
                            $group.bench_function(&bench_id, |b| {
                                b.iter(|| {
                                    let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                                    let mut w = BufBitWriter::<LE, _>::new(
                                        MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                                    );
                                    for &value in data_ref {
                                        black_box(w.$write_method(value, k).unwrap());
                                    }
                                });
                            });
                        }
                    }

                    if env_filter("BENCH_OPS", "read") {
                        if env_filter("BENCH_ENDIAN", "BE") {
                            let encoded = {
                                let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                                {
                                    let mut w = BufBitWriter::<BE, _>::new(
                                        MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                                    );
                                    for &value in &data {
                                        w.$write_method(value, k).unwrap();
                                    }
                                }
                                buffer
                            };
                            let n = data.len();
                            let bench_id = format!("{}/BE/{}/read", name_str, dist_name);
                            $group.bench_function(&bench_id, |b| {
                                b.iter(|| {
                                    let slice: &[ReadWord] =
                                        unsafe { encoded.align_to::<ReadWord>().1 };
                                    let mut r = BufBitReader::<BE, _>::new(
                                        MemWordReader::<ReadWord, _>::new(slice),
                                    );
                                    for _ in 0..n {
                                        black_box(r.$read_method(rk).unwrap());
                                    }
                                });
                            });
                        }
                        if env_filter("BENCH_ENDIAN", "LE") {
                            let encoded = {
                                let mut buffer: Vec<WriteWord> = Vec::with_capacity(N);
                                {
                                    let mut w = BufBitWriter::<LE, _>::new(
                                        MemWordWriterVec::<WriteWord, _>::new(&mut buffer),
                                    );
                                    for &value in &data {
                                        w.$write_method(value, k).unwrap();
                                    }
                                }
                                buffer
                            };
                            let n = data.len();
                            let bench_id = format!("{}/LE/{}/read", name_str, dist_name);
                            $group.bench_function(&bench_id, |b| {
                                b.iter(|| {
                                    let slice: &[ReadWord] =
                                        unsafe { encoded.align_to::<ReadWord>().1 };
                                    let mut r = BufBitReader::<LE, _>::new(
                                        MemWordReader::<ReadWord, _>::new(slice),
                                    );
                                    for _ in 0..n {
                                        black_box(r.$read_method(rk).unwrap());
                                    }
                                });
                            });
                        }
                    }
                }
            }
        }
    };
}

/// Comparative benchmarks: all codes compared side by side.
fn bench_comparative(c: &mut Criterion) {
    #[cfg(target_os = "linux")]
    benchmarks::utils::pin_to_core(5);

    let mut group = c.benchmark_group("comparative");
    group.throughput(Throughput::Elements(N as u64));

    // Fixed-parameter codes
    bench_comp!(group, "unary", write_unary, read_unary, |x: u64| x as usize + 1);
    bench_comp!(group, "gamma", write_gamma, read_gamma, len_gamma);
    bench_comp!(group, "delta", write_delta, read_delta, len_delta);
    bench_comp!(group, "omega", write_omega, read_omega, len_omega);
    bench_comp!(group, "vbyte_be", write_vbyte_be, read_vbyte_be, bit_len_vbyte);
    bench_comp!(group, "vbyte_le", write_vbyte_le, read_vbyte_le, bit_len_vbyte);

    // Specialized (table-using) variants
    bench_comp!(group, "zeta3", write_zeta3, read_zeta3, |x| len_zeta(x, 3));
    bench_comp!(group, "pi2", write_pi2, read_pi2, |x| len_pi(x, 2));

    // Parametric codes with k = 2..4 or 2..5
    for k in 2..4usize {
        bench_comp_k!(group, format!("zeta_{}", k),
            write_zeta(k), read_zeta(k), |x| len_zeta(x, k));
    }
    for k in 2..5usize {
        bench_comp_k!(group, format!("pi_{}", k),
            write_pi(k), read_pi(k), |x| len_pi(x, k));
        bench_comp_k!(group, format!("rice_{}", k),
            write_rice(k), read_rice(k), |x| len_rice(x, k));
        bench_comp_k!(group, format!("exp_golomb_{}", k),
            write_exp_golomb(k), read_exp_golomb(k), |x| len_exp_golomb(x, k));
        bench_comp_k!(group, format!("golomb_{}", k),
            write_golomb(k as u64), read_golomb(k as u64), |x| len_golomb(x, k as u64));
    }

    group.finish();
}

criterion_group! {
    name = comparative;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(std::time::Duration::from_secs(5))
        .measurement_time(std::time::Duration::from_secs(10));
    targets = bench_comparative
}

criterion_main!(comparative);
