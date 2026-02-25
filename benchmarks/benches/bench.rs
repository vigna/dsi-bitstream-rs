/*
 * SPDX-FileCopyrightText: 2025 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Unified Criterion benchmark suite for dsi-bitstream codes.
//!
//! Benchmarks are organized into Criterion groups with hierarchical naming:
//!   {code}/{endianness}/{distribution}/{operation}
//!
//! Environment variables for filtering:
//!   BENCH_CODES=gamma,delta    (default: all)
//!   BENCH_ENDIAN=BE            (default: both)
//!   BENCH_DIST=implied         (default: both)
//!   BENCH_OPS=read_buff,write  (default: all)

use benchmarks::data::*;
use benchmarks::N;
use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::prelude::*;
use std::hint::black_box;

// CHANGED: Word types are now determined by features, same as before, but
// organized more clearly at the top of the file.

#[cfg(feature = "u16")]
type ReadWord = u16;
#[cfg(feature = "u32")]
type ReadWord = u32;
#[cfg(feature = "u64")]
type ReadWord = u64;

// Write always uses u64 for reads mode, otherwise the selected word
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


// ─── Table-sweep benchmarks (used by bench_code_tables_*.py) ────────────────

/// Macro to register table-sweep benchmarks for a single code.
/// Generates separate closures for BufBitReader and BitReader (unbuffered)
/// since they are different types.
macro_rules! bench_code_tables {
    (
        $c:expr, $code_name:literal,
        $read_fn:ident, $write_fn:ident, $gen_data:ident,
        $($table_param:expr),*
    ) => {{
        let table_str = if ($($table_param),*,).0 { "Table" } else { "NoTable" };
        let univ = cfg!(feature = "univ");
        let (ratio, data) = $gen_data(univ);

        // Print hit ratio to stderr for the Python scripts to capture
        eprintln!("RATIO:{}::BE::{},{:.6}", $code_name, table_str, ratio);
        eprintln!("RATIO:{}::LE::{},{:.6}", $code_name, table_str, ratio);

        #[cfg(not(feature = "reads"))]
        {
            // Write benchmark — BE
            {
                let bench_id = format!("{}::BE::{}/write", $code_name, table_str);
                let data_ref = &data;
                $c.bench_function(&bench_id, |b| {
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
                let data_ref = &data;
                $c.bench_function(&bench_id, |b| {
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
                    for &value in &data {
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
                    for &value in &data {
                        w.$write_fn::<$($table_param),*>(value).unwrap();
                    }
                }
                buffer
            };

            let n = data.len();

            // Buffered read — BE
            {
                let bench_id = format!("{}::BE::{}/read_buff", $code_name, table_str);
                $c.bench_function(&bench_id, |b| {
                    b.iter(|| {
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
                let bench_id = format!("{}::LE::{}/read_buff", $code_name, table_str);
                $c.bench_function(&bench_id, |b| {
                    b.iter(|| {
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
                let bench_id = format!("{}::BE::{}/read_unbuff", $code_name, table_str);
                $c.bench_function(&bench_id, |b| {
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
                let bench_id = format!("{}::LE::{}/read_unbuff", $code_name, table_str);
                $c.bench_function(&bench_id, |b| {
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

    #[cfg(not(feature = "delta_gamma"))]
    {
        bench_code_tables!(c, "gamma", read_gamma_param, write_gamma_param, gen_gamma_data, true);
        bench_code_tables!(c, "gamma", read_gamma_param, write_gamma_param, gen_gamma_data, false);
        bench_code_tables!(c, "zeta3", read_zeta3_param, write_zeta3_param, gen_zeta3_data, true);
        bench_code_tables!(c, "zeta3", read_zeta3_param, write_zeta3_param, gen_zeta3_data, false);
        bench_code_tables!(c, "pi2", read_pi2_param, write_pi2_param, gen_pi2_data, true);
        bench_code_tables!(c, "pi2", read_pi2_param, write_pi2_param, gen_pi2_data, false);
        bench_code_tables!(c, "omega", read_omega_param, write_omega_param, gen_omega_data, true);
        bench_code_tables!(c, "omega", read_omega_param, write_omega_param, gen_omega_data, false);
        // delta without gamma tables
        bench_code_tables!(c, "delta", read_delta_param, write_delta_param, gen_delta_data, true, false);
        bench_code_tables!(c, "delta", read_delta_param, write_delta_param, gen_delta_data, false, false);
    }

    #[cfg(feature = "delta_gamma")]
    {
        // delta with gamma tables
        bench_code_tables!(c, "delta_gamma", read_delta_param, write_delta_param, gen_delta_data, true, true);
        bench_code_tables!(c, "delta_gamma", read_delta_param, write_delta_param, gen_delta_data, false, true);
    }
}

// ─── Comparative benchmarks ─────────────────────────────────────────────────

/// Macro to register a comparative benchmark for a code (both endiannesses,
/// both distributions, read + write).
macro_rules! bench_comp {
    ($c:expr, $name:literal, $write_method:ident, $read_method:ident, $len_fn:expr) => {
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
                        $c.bench_function(&bench_id, |b| {
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
                        $c.bench_function(&bench_id, |b| {
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
                        $c.bench_function(&bench_id, |b| {
                            b.iter(|| {
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
                        $c.bench_function(&bench_id, |b| {
                            b.iter(|| {
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
    ($c:expr, $name:expr, $write_method:ident($k:expr), $read_method:ident($rk:expr), $len_fn:expr) => {
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
                            $c.bench_function(&bench_id, |b| {
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
                            $c.bench_function(&bench_id, |b| {
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
                            $c.bench_function(&bench_id, |b| {
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
                            $c.bench_function(&bench_id, |b| {
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

    // Fixed-parameter codes
    bench_comp!(c, "unary", write_unary, read_unary, |x: u64| x as usize + 1);
    bench_comp!(c, "gamma", write_gamma, read_gamma, len_gamma);
    bench_comp!(c, "delta", write_delta, read_delta, len_delta);
    bench_comp!(c, "omega", write_omega, read_omega, len_omega);
    bench_comp!(c, "vbyte_be", write_vbyte_be, read_vbyte_be, bit_len_vbyte);
    bench_comp!(c, "vbyte_le", write_vbyte_le, read_vbyte_le, bit_len_vbyte);

    // Specialized (table-using) variants
    bench_comp!(c, "zeta3", write_zeta3, read_zeta3, |x| len_zeta(x, 3));
    bench_comp!(c, "pi2", write_pi2, read_pi2, |x| len_pi(x, 2));

    // Parametric codes with k = 2..4 or 2..5
    for k in 2..4usize {
        bench_comp_k!(c, format!("zeta_{}", k),
            write_zeta(k), read_zeta(k), |x| len_zeta(x, k));
    }
    for k in 2..5usize {
        bench_comp_k!(c, format!("pi_{}", k),
            write_pi(k), read_pi(k), |x| len_pi(x, k));
        bench_comp_k!(c, format!("rice_{}", k),
            write_rice(k), read_rice(k), |x| len_rice(x, k));
        bench_comp_k!(c, format!("exp_golomb_{}", k),
            write_exp_golomb(k), read_exp_golomb(k), |x| len_exp_golomb(x, k));
        bench_comp_k!(c, format!("golomb_{}", k),
            write_golomb(k as u64), read_golomb(k as u64), |x| len_golomb(x, k as u64));
    }
}

criterion_group! {
    name = tables;
    config = Criterion::default()
        .sample_size(30)
        .warm_up_time(std::time::Duration::from_secs(3))
        .measurement_time(std::time::Duration::from_secs(5));
    targets = bench_tables
}

criterion_group! {
    name = comparative;
    config = Criterion::default()
        .sample_size(50)
        .warm_up_time(std::time::Duration::from_secs(5))
        .measurement_time(std::time::Duration::from_secs(10));
    targets = bench_comparative
}

criterion_main!(tables, comparative);
