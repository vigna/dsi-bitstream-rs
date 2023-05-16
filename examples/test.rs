use dsi_bitstream::prelude::*;
use rand::Rng;
use std::hint::black_box;

type ReadWord = u32;
type BufferWord = u64;
/// How many random codes we will write and read in the benchmark
const VALUES: usize = 1_000_000;
/// How many iterations to do before starting measuring, this is done to warmup
/// the caches and the branch predictor
const WARMUP_ITERS: usize = 100;
/// How many iterations of measurement we will execute
const BENCH_ITERS: usize = 11;
/// For how many times we will measure the measurement overhead
const CALIBRATION_ITERS: usize = 100_000;
/// To proprly test delta we compute a discrete version of the indended
/// distribution. The original distribution is infinite but we need to cut it
/// down to a finite set. This value represent the maximum value we are going to
/// extract
const DELTA_DISTR_SIZE: usize = 1_000_000;

use std::time::Instant;

fn main() {
    let mut buffer = Vec::with_capacity(VALUES);
    let mut rng = rand::thread_rng();

    let distr = rand_distr::Zeta::new(2.0).unwrap();
    let data = (0..VALUES)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>();

    buffer.clear();
    {
        // init the writer
        let mut r = BufferedBitStreamWrite::<M2L, _>::new(MemWordWriteVec::new(&mut buffer));
        for &value in &data {
            black_box(r.write_gamma::<false>(value).unwrap());
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
    for _ in &data {
        black_box(r.read_gamma::<true>().unwrap());
    }
    let nanos = r_start.elapsed().as_nanos();
    println!("{}", nanos);

    let mut r = UnbufferedBitStreamRead::<M2L, _>::new(MemWordReadInfinite::new(&buffer));
    // measure
    let r_start = Instant::now();
    for _ in &data {
        black_box(r.read_gamma::<true>().unwrap());
    }
    let nanos = r_start.elapsed().as_nanos();
    println!("{}", nanos);
}
