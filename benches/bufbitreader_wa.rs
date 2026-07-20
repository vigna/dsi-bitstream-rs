/*
 * Temporary evaluation bench: BufBitReader over a non-peekable WordAdapter
 * (std::io::Cursor), so read_word_opt() returns None and the two-word top-up is
 * compiled out. Mirrors benches/bufbitreader.rs workloads (u32 read word).
 */

mod common;

use common::N;
use common::data::gen_data;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use std::hint::black_box;
use std::io::Cursor;

type ReadWord = u32;

fn to_bytes(words: &[u64]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_ne_bytes()).collect()
}

macro_rules! bench_endian_wa {
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
            let bytes = to_bytes(&encoded);

            let expected = data.iter().fold(0u64, |acc, &v| acc.wrapping_add(v));
            let mut reader = BufBitReader::<$E, _>::new(WordAdapter::<ReadWord, _>::new(
                Cursor::new(bytes.as_slice()),
            ));
            let mut sum = 0u64;
            for _ in 0..data.len() {
                sum = sum.wrapping_add(reader.read_unary().unwrap());
            }
            assert_eq!(sum, expected, "read_unary/{} checksum mismatch", $ename);

            $group.bench_function(concat!("read_unary/", $ename), |b| {
                b.iter(|| {
                    let mut r = BufBitReader::<$E, _>::new(WordAdapter::<ReadWord, _>::new(
                        Cursor::new(bytes.as_slice()),
                    ));
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
            let bytes = to_bytes(&encoded);

            let expected = values.iter().fold(0u64, |acc, &v| acc.wrapping_add(v));
            let mut reader = BufBitReader::<$E, _>::new(WordAdapter::<ReadWord, _>::new(
                Cursor::new(bytes.as_slice()),
            ));
            let mut sum = 0u64;
            for &n_bits in widths {
                sum = sum.wrapping_add(reader.read_bits(n_bits).unwrap());
            }
            assert_eq!(sum, expected, "read_bits/{} checksum mismatch", $ename);

            $group.bench_function(concat!("read_bits/", $ename), |b| {
                b.iter(|| {
                    let mut r = BufBitReader::<$E, _>::new(WordAdapter::<ReadWord, _>::new(
                        Cursor::new(bytes.as_slice()),
                    ));
                    for &n_bits in widths {
                        black_box(r.read_bits(n_bits).unwrap());
                    }
                });
            });
        }
    }};
}

fn bench_bufbitreader_wa(c: &mut Criterion) {
    #[cfg(target_os = "linux")]
    common::utils::pin_to_core(5);

    let mut group = c.benchmark_group("bufbitreader_wa");
    group.throughput(Throughput::Elements(u64::try_from(N).unwrap()));

    let unary_data = gen_data(
        |x| usize::try_from(x.saturating_add(1)).expect("unary code length fits usize"),
        false,
    );

    let mut rng = SmallRng::seed_from_u64(0x0DD5_EED5);
    let widths: Vec<usize> = (0..N).map(|_| rng.random_range(1..=32usize)).collect();
    let values: Vec<u64> = widths
        .iter()
        .map(|&w| rng.random::<u64>() & (u64::MAX >> (64 - w)))
        .collect();

    bench_endian_wa!(group, BE, "BE", &unary_data, &widths, &values);
    bench_endian_wa!(group, LE, "LE", &unary_data, &widths, &values);

    group.finish();
}

criterion_group! {
    name = bufbitreader_wa;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = bench_bufbitreader_wa
}

criterion_main!(bufbitreader_wa);
