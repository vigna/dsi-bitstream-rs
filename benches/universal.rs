//! Here we test 5 different choices:
//! - continuation bits at the start vs spread on each byte
//! - bits vs bytes streams
//! - little endian vs big endian
//! - 1 or 0 as continuation bit
//! - bijective or not
//!
//! - if continuation bits at start, test if chains vs leading_ones and jump table
//!
use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::codes::{LevenshteinRead, LevenshteinWrite, OmegaRead, OmegaWrite};
use dsi_bitstream::impls::{BufBitReader, BufBitWriter, MemWordReader, MemWordWriterVec};
use dsi_bitstream::traits::{BitRead, BitWrite, Endianness, BE, LE};
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::hint::black_box;
use std::time::Duration;

const GAMMA_DATA: usize = 1000000;
const CAPACITY: usize = GAMMA_DATA * 4;

fn gen_data(n: usize) -> Vec<u64> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distr = rand_distr::Zeta::new(1.1).unwrap();

    (0..n)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>()
}

fn bench_levenshtein(c: &mut Criterion) {
    bench_levenshtein_endianness::<BE>(c);
    bench_levenshtein_endianness::<LE>(c);
}

fn bench_levenshtein_endianness<E: Endianness>(c: &mut Criterion)
where
    for<'a> BufBitReader<E, MemWordReader<u32, &'a [u32]>>: BitRead<E>,
    for<'a> BufBitWriter<E, MemWordWriterVec<u64, &'a mut Vec<u64>>>: BitWrite<E>,
{
    let mut v = <Vec<u64>>::with_capacity(CAPACITY);
    let s = gen_data(GAMMA_DATA);

    c.bench_function(
        &format!("universal: Levenshtein ({}-endian, write)", E::NAME),
        |b| {
            b.iter(|| {
                let mut w = <BufBitWriter<E, _>>::new(MemWordWriterVec::new(&mut v));
                for &t in &s {
                    black_box(w.write_levenshtein(t).unwrap());
                }
            })
        },
    );

    let v = unsafe { v.align_to::<u32>().1 };

    c.bench_function(
        &format!("universal: Levenshtein ({}-endian, read)", E::NAME),
        |b| {
            b.iter(|| {
                let mut r = BufBitReader::<E, _>::new(MemWordReader::new(v));
                for _ in &s {
                    black_box(r.read_levenshtein().unwrap());
                }
            })
        },
    );
}

fn bench_omega(c: &mut Criterion) {
    bench_omega_endianness::<BE>(c);
    bench_omega_endianness::<LE>(c);
}

fn bench_omega_endianness<E: Endianness>(c: &mut Criterion)
where
    for<'a> BufBitReader<E, MemWordReader<u32, &'a [u32]>>: BitRead<E>,
    for<'a> BufBitWriter<E, MemWordWriterVec<u64, &'a mut Vec<u64>>>: BitWrite<E>,
{
    let mut v = <Vec<u64>>::with_capacity(CAPACITY);

    let s = gen_data(GAMMA_DATA);

    c.bench_function(
        &format!("universal: omega ({}-endian, write)", E::NAME),
        |b| {
            b.iter(|| {
                let mut w = <BufBitWriter<E, _>>::new(MemWordWriterVec::new(&mut v));
                for &t in &s {
                    black_box(w.write_omega(t).unwrap());
                }
            })
        },
    );

    let v = unsafe { v.align_to::<u32>().1 };

    c.bench_function(
        &format!("universal: omega ({}-endian, read)", E::NAME),
        |b| {
            b.iter(|| {
                let mut r = BufBitReader::<E, _>::new(MemWordReader::new(v));
                for _ in &s {
                    black_box(r.read_omega().unwrap());
                }
            })
        },
    );
}

fn benchmark(c: &mut Criterion) {
    bench_levenshtein(c);
    bench_omega(c);
}

criterion_group! {
    name = universal_benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = benchmark
}
criterion_main!(universal_benches);
