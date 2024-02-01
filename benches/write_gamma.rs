use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;
use std::hint::black_box;
use std::time::Duration;

pub fn criterion_benchmark(c: &mut Criterion) {
    let v_le = <Vec<u64>>::with_capacity(2000000000);
    let mut w_le = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(v_le));
    let v_be = <Vec<u64>>::with_capacity(2000000000);
    let mut w_be = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(v_be));
    let mut r = SmallRng::seed_from_u64(0);

    c.bench_function("rng + trailing_zeros", |b| {
        b.iter(|| black_box(r.next_u64().trailing_zeros() as u64))
    });

    c.bench_function("write_gamma<BE> (no tables)", |b| {
        b.iter(|| w_be.write_gamma_param::<false>(black_box(r.next_u64().trailing_zeros() as u64)))
    });

    c.bench_function("write_gamma<LE> (no tables)", |b| {
        b.iter(|| w_le.write_gamma_param::<false>(black_box(r.next_u64().trailing_zeros() as u64)))
    });

    c.bench_function("write_gamma<BE> (tables)", |b| {
        b.iter(|| w_be.write_gamma_param::<true>(black_box(r.next_u64().trailing_zeros() as u64)))
    });

    c.bench_function("write_gamma<LE> (tables)", |b| {
        b.iter(|| w_le.write_gamma_param::<true>(black_box(r.next_u64().trailing_zeros() as u64)))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = criterion_benchmark
}
criterion_main!(benches);
