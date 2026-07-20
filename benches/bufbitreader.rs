/*
 * SPDX-FileCopyrightText: 2026 Tommaso Fontana
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Focused Criterion benchmarks for [`BufBitReader`]: raw `read_unary` and
//! `read_bits` calls, big and little endian.
//!
//! The `comparative` benchmark exercises `read_unary` directly (via the unary
//! code) but `read_bits` only indirectly through higher-level codes. These
//! benchmarks measure the two methods in isolation so that changes to
//! [`BufBitReader`] are visible without code-level overhead.
//!
//! Data is deterministic: unary values are sampled from the implied
//! distribution p(x) = 2⁻⁽ˣ⁺¹⁾ with a fixed seed (as in `comparative`), and
//! `read_bits` widths are uniform in 1..=32 with matching masked values.
//! Before each benchmark is registered, one decode pass verifies a
//! wrapping-sum checksum against the source values, so the run fails on any
//! change that breaks decoding.
//!
//! ```bash
//! cargo bench --bench bufbitreader --features implied
//! ```

mod common;

use common::N;
use common::data::gen_data;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use std::hint::black_box;

#[cfg(feature = "bench-u16")]
type ReadWord = u16;
#[cfg(all(feature = "bench-u64", not(feature = "bench-u16")))]
type ReadWord = u64;
#[cfg(not(any(feature = "bench-u16", feature = "bench-u64")))]
type ReadWord = u32;

/// Criterion group name, disambiguated by read-word size so that runs with
/// different `bench-*` word features do not overwrite each other's results.
#[cfg(feature = "bench-u16")]
const GROUP: &str = "bufbitreader_u16";
#[cfg(all(feature = "bench-u64", not(feature = "bench-u16")))]
const GROUP: &str = "bufbitreader_u64";
#[cfg(not(any(feature = "bench-u16", feature = "bench-u64")))]
const GROUP: &str = "bufbitreader";

/// Registers `read_unary` and `read_bits` benchmarks for one endianness.
///
/// Values are pre-encoded into a `Box<[u64]>` to guarantee alignment for
/// reinterpretation as `&[ReadWord]` via `align_to`, as in `comparative`.
macro_rules! bench_endian {
    ($group:expr, $E:ty, $ename:literal, $unary_data:expr, $widths:expr, $values:expr) => {{
        // read_unary
        {
            let data: &[u64] = $unary_data;
            let encoded = {
                let mut buffer: Box<[u64]> = vec![0u64; 10 * N].into_boxed_slice();
                {
                    let mut wr =
                        BufBitWriter::<$E, _>::new(MemWordWriterSlice::<u64, _>::new(&mut *buffer));
                    for &value in data {
                        wr.write_unary(value).unwrap();
                    }
                }
                buffer
            };
            // SAFETY: Box<[u64]> is aligned to 8 bytes, which satisfies
            // alignment for ReadWord (u16/u32/u64).
            let slice: &[ReadWord] = unsafe { encoded.align_to::<ReadWord>().1 };

            // Correctness guard: one decode pass must reproduce the input sum.
            let expected = data.iter().fold(0u64, |acc, &v| acc.wrapping_add(v));
            let mut reader = BufBitReader::<$E, _>::new(MemWordReader::new(slice));
            let mut sum = 0u64;
            for _ in 0..data.len() {
                sum = sum.wrapping_add(reader.read_unary().unwrap());
            }
            assert_eq!(sum, expected, "read_unary/{} checksum mismatch", $ename);

            $group.bench_function(concat!("read_unary/", $ename), |b| {
                b.iter(|| {
                    let mut r = BufBitReader::<$E, _>::new(MemWordReader::new(slice));
                    for _ in 0..N {
                        black_box(r.read_unary().unwrap());
                    }
                });
            });
        }
        // read_bits
        {
            let widths: &[usize] = $widths;
            let values: &[u64] = $values;
            let encoded = {
                let mut buffer: Box<[u64]> = vec![0u64; 10 * N].into_boxed_slice();
                {
                    let mut wr =
                        BufBitWriter::<$E, _>::new(MemWordWriterSlice::<u64, _>::new(&mut *buffer));
                    for (&value, &n_bits) in values.iter().zip(widths) {
                        wr.write_bits(value, n_bits).unwrap();
                    }
                }
                buffer
            };
            // SAFETY: Box<[u64]> is aligned to 8 bytes, which satisfies
            // alignment for ReadWord (u16/u32/u64).
            let slice: &[ReadWord] = unsafe { encoded.align_to::<ReadWord>().1 };

            // Correctness guard: one decode pass must reproduce the input sum.
            let expected = values.iter().fold(0u64, |acc, &v| acc.wrapping_add(v));
            let mut reader = BufBitReader::<$E, _>::new(MemWordReader::new(slice));
            let mut sum = 0u64;
            for &n_bits in widths {
                sum = sum.wrapping_add(reader.read_bits(n_bits).unwrap());
            }
            assert_eq!(sum, expected, "read_bits/{} checksum mismatch", $ename);

            $group.bench_function(concat!("read_bits/", $ename), |b| {
                b.iter(|| {
                    let mut r = BufBitReader::<$E, _>::new(MemWordReader::new(slice));
                    for &n_bits in widths {
                        black_box(r.read_bits(n_bits).unwrap());
                    }
                });
            });
        }
    }};
}

fn bench_bufbitreader(c: &mut Criterion) {
    #[cfg(target_os = "linux")]
    common::utils::pin_to_core(2);

    let mut group = c.benchmark_group(GROUP);
    // Lossless: N = 1_000_000 fits u64 on all supported targets.
    group.throughput(Throughput::Elements(u64::try_from(N).unwrap()));

    // Unary values from the implied distribution p(x) = 2⁻⁽ˣ⁺¹⁾ (fixed seed
    // inside gen_data); saturating_add keeps the length closure total.
    let unary_data = gen_data(
        |x| usize::try_from(x.saturating_add(1)).expect("unary code length fits usize"),
        false,
    );

    // Widths uniform in 1..=32 (exercises both the buffered path and the
    // refill path for u16/u32 read words) with values masked to their width.
    let mut rng = SmallRng::seed_from_u64(0x0DD5_EED5);
    let widths: Vec<usize> = (0..N).map(|_| rng.random_range(1..=32usize)).collect();
    let values: Vec<u64> = widths
        .iter()
        .map(|&w| rng.random::<u64>() & (u64::MAX >> (64 - w)))
        .collect();

    bench_endian!(group, BE, "BE", &unary_data, &widths, &values);
    bench_endian!(group, LE, "LE", &unary_data, &widths, &values);

    group.finish();
}

criterion_group! {
    name = bufbitreader;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = bench_bufbitreader
}

criterion_main!(bufbitreader);
