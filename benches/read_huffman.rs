use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
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
    const BITS: usize = 12;
    let mut v_le = <Vec<u64>>::with_capacity(128);
    let mut v_le2 = <Vec<u64>>::with_capacity(128);
    let mut w_le = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(&mut v_le));
    let mut h_le = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(&mut v_le2));
    
    let mut v_be = <Vec<u64>>::with_capacity(128);
    let mut v_be2 = <Vec<u64>>::with_capacity(128);
    let mut w_be = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut v_be));
    let mut h_be = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut v_be2));
    
    let s = gen_gamma_data(1 << BITS);

    println!("Generated data");
    let mut counts=  vec![];
    for v in s.iter() {
        if counts.len() <= *v as usize {
            counts.resize(*v as usize + 1, 0);
        }
        counts[*v as usize] += 1;
    }
    println!("Built counts");

    let huffman = huffman::HuffmanTree::new(&counts).unwrap();
    
    println!("Built table");

    for v in s.iter() {
        w_le.write_gamma(*v).unwrap();
        w_be.write_gamma(*v).unwrap();
        huffman.encode(*v, &mut h_le).unwrap();
        huffman.encode(*v, &mut h_be).unwrap();
    }

    println!("Encoded data");

    drop(w_le);
    drop(w_be);
    drop(h_le);
    drop(h_be);

    let mut r_be = <BufBitReader<BE, _>>::new(MemWordReader::new(v_be));
    let mut h_be = <BufBitReader<BE, _>>::new(MemWordReader::new(v_be2));
    let mut r_le = <BufBitReader<LE, _>>::new(MemWordReader::new(v_le));
    let mut h_le = <BufBitReader<LE, _>>::new(MemWordReader::new(v_le2));

    let mut i = 0;

    c.bench_function("read_gamma<BE>", |b| {
        b.iter(|| {
            black_box(r_be.read_gamma().unwrap());
            i += 1;
            if i == 1 << BITS {
                i = 0;
                r_be.set_bit_pos(0).unwrap();
            }
        })
    });

    i = 0;
    c.bench_function("read_huffman<BE>", |b| {
        b.iter(|| {
            black_box(huffman.decode(&mut h_be).unwrap());
            i += 1;
            if i == 1 << BITS {
                i = 0;
                r_be.set_bit_pos(0).unwrap();
            }
        })
    });

    i = 0;
    c.bench_function("read_huffman<LE>", |b| {
        b.iter(|| {
            black_box(huffman.decode(&mut h_le).unwrap());
            i += 1;
            if i == 1 << BITS {
                i = 0;
                r_be.set_bit_pos(0).unwrap();
            }
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = criterion_benchmark
}
criterion_main!(benches);
