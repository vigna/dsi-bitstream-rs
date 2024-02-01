use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr;
use std::hint::black_box;
use std::time::Duration;

pub fn gen_gamma_data(n: usize) -> Vec<u64> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distr = rand_distr::Zeta::new(2.0).unwrap();

    (0..n)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let v_le = <Vec<u64>>::with_capacity(2000000000);
    let mut w_le = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(v_le));
    let v_be = <Vec<u64>>::with_capacity(2000000000);
    let mut w_be = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(v_be));
    let s = gen_gamma_data(1 << 24);
    let mut i = 0;

    c.bench_function("write_gamma<BE> (no tables)", |b| {
        b.iter(|| {
            w_be.write_gamma_param::<false>(s[i % (1 << 24)]).unwrap();
            i += 1;
        })
    });

    c.bench_function("write_gamma<LE> (no tables)", |b| {
        b.iter(|| {
            w_le.write_gamma_param::<false>(s[i % (1 << 24)]).unwrap();
            i += 1;
        })
    });

    c.bench_function("write_gamma<BE> (tables)", |b| {
        b.iter(|| {
            w_be.write_gamma_param::<true>(s[i % (1 << 24)]).unwrap();
            i += 1;
        })
    });

    c.bench_function("write_gamma<LE> (tables)", |b| {
        b.iter(|| {
            w_le.write_gamma_param::<true>(s[i % (1 << 24)]).unwrap();
            i += 1;
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = criterion_benchmark
}
criterion_main!(benches);
