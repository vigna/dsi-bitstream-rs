use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::hint::black_box;
use std::rc::Rc;
use std::time::Duration;

pub fn gen_gamma_data(n: usize) -> Vec<u64> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distr = rand_distr::Zeta::new(2.0).unwrap();

    (0..n)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let data = <Vec<u64>>::with_capacity(2000000000);
    let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(data));
    let s = gen_gamma_data(1 << 27);

    for value in s {
        writer.write_gamma(value).unwrap();
    }

    let data = writer.into_inner().unwrap().into_inner();
    let data_ref = &data;

    c.bench_function("read_gamma CabaBitReader", |b| {
        let data = unsafe{data.align_to().1}.to_vec();   
        let data: Rc<[u8]> = data.into_boxed_slice().into();
        let mut reader = CabaBinaryReader::new(data);

        b.iter(|| {
            black_box(reader.read_gamma().unwrap());
        });
    });

    c.bench_function("read_gamma BitReader", |b| {
        let mut reader = BitReader::<BE, _>::new(MemWordReader::<u64, _>::new(data_ref));

        b.iter(|| {
            black_box(reader.read_gamma().unwrap());
        })
    });

    c.bench_function("read_gamma BufBitReader", |b| {
        let mut reader = BufBitReader::<BE, _>::new(MemWordReader::<u32, _>::new(unsafe{data_ref.align_to().1}));

        b.iter(|| {
            black_box(reader.read_gamma().unwrap());
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = criterion_benchmark
}
criterion_main!(benches);
