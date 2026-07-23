/*
 * SPDX-FileCopyrightText: 2026 Tommaso Fontana
 * SPDX-FileCopyrightText: 2026 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Composite Criterion benchmark for [`BufBitReader`]: `read_gamma` with
//! default parameters (reading tables enabled), big and little endian.
//!
//! Kept in a separate binary from `bufbitreader` so that adding composite
//! cases does not perturb the code layout of the four synthetic anchor
//! cases (a 24% swing on `read_bits/BE` was observed when both case sets
//! shared one binary).
//!
//! Gamma values are sampled from the implied distribution with a fixed seed
//! (as in `comparative`); a wrapping-sum checksum is verified before timing.
//!
//! ```bash
//! cargo bench --bench bufbitreader_gamma --features implied
//! ```

mod common;

use common::N;
use common::data::gen_gamma_data;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dsi_bitstream::prelude::*;
use std::hint::black_box;

#[cfg(feature = "bench-u16")]
type ReadWord = u16;
#[cfg(all(feature = "bench-u64", not(feature = "bench-u16")))]
type ReadWord = u64;
#[cfg(not(any(feature = "bench-u16", feature = "bench-u64")))]
type ReadWord = u32;

/// Registers the `read_gamma` benchmark for one endianness.
///
/// Values are pre-encoded into a `Box<[u64]>` to guarantee alignment for
/// reinterpretation as `&[ReadWord]` via `align_to`, as in `comparative`.
macro_rules! bench_endian {
    ($group:expr, $E:ty, $ename:literal, $gamma_data:expr) => {{
        let data: &[u64] = $gamma_data;
        let encoded = {
            let mut buffer: Box<[u64]> = vec![0u64; 10 * N].into_boxed_slice();
            {
                let mut wr =
                    BufBitWriter::<$E, _>::new(MemWordWriterSlice::<u64, _>::new(&mut *buffer));
                for &value in data {
                    wr.write_gamma(value).unwrap();
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
            sum = sum.wrapping_add(reader.read_gamma().unwrap());
        }
        assert_eq!(sum, expected, "read_gamma/{} checksum mismatch", $ename);

        $group.bench_function(concat!("read_gamma/", $ename), |b| {
            b.iter(|| {
                let mut r = BufBitReader::<$E, _>::new(MemWordReader::new(slice));
                for _ in 0..N {
                    black_box(r.read_gamma().unwrap());
                }
            });
        });
    }};
}

fn bench_bufbitreader_gamma(c: &mut Criterion) {
    #[cfg(target_os = "linux")]
    common::utils::pin_to_core(2);

    let mut group = c.benchmark_group("bufbitreader_gamma");
    // Lossless: N = 1_000_000 fits u64 on all supported targets.
    group.throughput(Throughput::Elements(u64::try_from(N).unwrap()));

    // Gamma values from the implied distribution (fixed seed inside
    // gen_data); decoded with default parameters (reading tables enabled).
    let (_, gamma_data) = gen_gamma_data(false);

    bench_endian!(group, BE, "BE", &gamma_data);
    bench_endian!(group, LE, "LE", &gamma_data);

    group.finish();
}

criterion_group! {
    name = bufbitreader_gamma;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = bench_bufbitreader_gamma
}

criterion_main!(bufbitreader_gamma);
