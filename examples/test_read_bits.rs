use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::hint::black_box;

type ReadWord = u32;
type BufferWord = u64;
/// How many random codes we will write and read in the benchmark
const VALUES: usize = 100_000_000;

use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <low> <high>", args[0]);
        return;
    }
    let low: usize = args[1].parse().unwrap();
    let high: usize = args[2].parse().unwrap();
    let mut buffer: Vec<u64> = Vec::with_capacity(VALUES);
    let mut rng = SmallRng::seed_from_u64(0);
    let mut data: Vec<usize> = Vec::with_capacity(VALUES);
    for _ in 0..VALUES {
        data.push(rng.gen_range(low..high) as usize);
    }

    for _ in 0..10 {
        // M2L
        println!("M2L");
        buffer.clear();
        {
            // init the writer
            let mut r = BufferedBitStreamWrite::<M2L, _>::new(MemWordWriteVec::new(&mut buffer));
            for &value in &data {
                black_box(r.write_bits(1, value).unwrap());
            }
        }

        let transmuted_buff: &[ReadWord] = unsafe {
            core::slice::from_raw_parts(
                buffer.as_ptr() as *const ReadWord,
                buffer.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
            )
        };

        // init the reader
        let mut r = BufferedBitStreamRead::<M2L, BufferWord, _>::new(MemWordReadInfinite::new(
            &transmuted_buff,
        ));
        // measure
        let r_start = Instant::now();
        for &w in &data {
            black_box(r.read_bits(w).unwrap());
        }
        let nanos = r_start.elapsed().as_nanos();
        println!("{}", nanos);

        // L2M
        println!("L2M");
        buffer.clear();
        {
            // init the writer
            let mut r = BufferedBitStreamWrite::<L2M, _>::new(MemWordWriteVec::new(&mut buffer));
            for &w in &data {
                black_box(r.write_bits(1, w).unwrap());
            }
        }

        let transmuted_buff: &[ReadWord] = unsafe {
            core::slice::from_raw_parts(
                buffer.as_ptr() as *const ReadWord,
                buffer.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
            )
        };

        // init the reader
        let mut r = BufferedBitStreamRead::<L2M, BufferWord, _>::new(MemWordReadInfinite::new(
            &transmuted_buff,
        ));

        // measure
        let r_start = Instant::now();
        for &w in &data {
            black_box(r.read_bits(w).unwrap());
        }
        let nanos = r_start.elapsed().as_nanos();
        println!("{}", nanos);
    }
}
