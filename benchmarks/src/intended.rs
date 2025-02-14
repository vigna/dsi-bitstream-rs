use dsi_bitstream::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::Distribution;
use std::hint::black_box;
use std::time::Instant;

pub mod metrics;
use metrics::*;

pub mod utils;
use utils::*;

pub mod find_change;
use find_change::*;

/// Number of read/write operations tested for each combination of parameters.
pub const N: usize = 5_000_000;
/// Number of warmup read/write operations.
pub const WARMUP_ITERS: usize = 7;
/// How many iterations of measurement we will execute.
pub const BENCH_ITERS: usize = 31;
/// For how many times we will measure the measurement overhead.
pub const CALIBRATION_ITERS: usize = 1_000_000;

type WriteWord = u64;
type ReadWord = u32;

fn bench<E: Endianness>(
    calibration: u128,
    code: impl AsRef<str>,
    write_fn: impl Fn(
        &mut BufBitWriter<E, MemWordWriterVec<WriteWord, &mut Vec<WriteWord>>>,
        u64,
    ) -> usize,
    read_fn: impl Fn(&mut BufBitReader<E, MemWordReader<ReadWord, &[ReadWord]>>) -> u64,
    len: impl Fn(u64) -> usize,
) where
    for<'a> BufBitWriter<E, MemWordWriterVec<WriteWord, &'a mut Vec<WriteWord>>>: BitWrite<E>,
    for<'a> BufBitReader<E, MemWordReader<ReadWord, &'a [ReadWord]>>: BitRead<E>,
{

    let data = gen_data(&len, N);

    // calculate the total number of bits we will write
    // so we can guarantee that the buffer always has enough capacity
    let total_bits = data.iter().map(|&x| len(x) as u64).sum::<u64>();
    let mut buffer: Vec<WriteWord> = Vec::with_capacity(total_bits.div_ceil(64) as usize + 10);

    let mut read = MetricsStream::with_capacity(BENCH_ITERS);
    let mut write = MetricsStream::with_capacity(BENCH_ITERS);


    for iter in 0..(WARMUP_ITERS + BENCH_ITERS) {
        buffer.clear();

        {
            let mut w =
                BufBitWriter::<E, _>::new(MemWordWriterVec::<WriteWord, _>::new(&mut buffer));

            let w_start = Instant::now();

            for &value in &data {
                black_box(write_fn(&mut w, value));
            }

            if iter >= WARMUP_ITERS {
                write.update((w_start.elapsed().as_nanos() - calibration) as f64);
            }
        }

        {
            let slice = unsafe { buffer.align_to::<ReadWord>().1 };
            let mut r = BufBitReader::<E, _>::new(MemWordReader::<ReadWord, _>::new(slice));
            let r_start = Instant::now();
            for &value in &data {
                let res = black_box(read_fn(&mut r));
                debug_assert_eq!(res, value);
            }

            let nanos = r_start.elapsed().as_nanos();

            if iter >= WARMUP_ITERS {
                read.update((nanos - calibration) as f64);
            }
        }
    }

    let write = write.finalize();
    let read = read.finalize();

    println!(
        "{:<12}\twrite\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
        code.as_ref(),
        E::NAME,
        write.avg / N as f64,
        write.std / N as f64,
        write.percentile_25 / N as f64,
        write.median / N as f64,
        write.percentile_75 / N as f64,
    );
    println!(
        "{:<12}\tread\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
        code.as_ref(),
        E::NAME,
        read.avg / N as f64,
        read.std / N as f64,
        read.percentile_25 / N as f64,
        read.median / N as f64,
        read.percentile_75 / N as f64,
    );
}

/// Given the len function of a code, generate data following its intended
/// distribution, i.e. a code-word with length l has probability 2^(-l).
/// This code works only with monotonic non decreasing len functions.
fn gen_data(f: impl Fn(u64) -> usize, n_samples: usize) -> Vec<u64> {
    let change_points = FindChangePoints::new(f)
        .take_while(|(_input, len)| *len <= 128)
        .collect::<Vec<_>>();
    // convert to len probabilities
    let probabilities = change_points
        .windows(2)
        .map(|window| {
            let (input, len) = window[0];
            let (next_input, _next_len) = window[1];
            let prob = 2.0_f64.powi(-(len as i32));
            prob * (next_input - input) as f64
        })
        .collect::<Vec<_>>();
    // TODO!: this ignores the last change point

    let distr = 
    rand_distr::weighted::WeightedAliasIndex::new(probabilities).unwrap();
    let mut rng = SmallRng::seed_from_u64(0xbadc0ffee);

    (0..n_samples)
        .map(|_| {
            // sample a len with the correct probability
            let idx = distr.sample(&mut rng);
            // now we sample a random value with the sampled len
            let (start_input, _len) = change_points[idx];
            let (end_input, _len) = change_points[idx + 1];
            rng.random_range(start_input..end_input)
        })
        .collect::<Vec<_>>()
}


pub fn main() {
    // tricks to reduce the noise
    #[cfg(target_os = "linux")]
    pin_to_core(5);

    // figure out how much overhead we add by measuring
    let calibration = calibrate_overhead();
    println!("{:<12}\trw\tendianness\tavg\tstd\t25%\tmedian\t75%", "code");

    macro_rules! bench {
        ($name:expr, $write:expr, $read:expr, $len:expr,) => {
            bench::<LE>(
                calibration,
                $name,
                $write,
                $read,
                $len,
            );
            bench::<BE>(
                calibration,
                $name,
                $write,
                $read,
                $len,
            );
        };
    }

    bench!(
        "unary", 
        |w, x| w.write_unary(x).unwrap(),
        |r| r.read_unary().unwrap(),
        |x| x as usize + 1,
    );
    bench!(
        "gamma", 
        |w, x| w.write_gamma(x).unwrap(),
        |r| r.read_gamma().unwrap(),
        |x| len_gamma(x),
    );
    bench!(
        "delta", 
        |w, x| w.write_delta(x).unwrap(),
        |r| r.read_delta().unwrap(),
        |x| len_delta(x),
    );
    bench!(
        "omega", 
        |w, x| w.write_omega(x).unwrap(),
        |r| r.read_omega().unwrap(),
        |x| len_omega(x),
    );
    bench!(
        "vbyte_be", 
        |w, x| w.write_vbyte_be(x).unwrap(),
        |r| r.read_vbyte_be().unwrap(),
        |x| bit_len_vbyte(x),
    );
    bench!(
        "vbyte_le", 
        |w, x| w.write_vbyte_le(x).unwrap(),
        |r| r.read_vbyte_le().unwrap(),
        |x| bit_len_vbyte(x),
    );
    bench!(
        "zeta_3_table", 
        |w, x| w.write_zeta3(x).unwrap(),
        |r| r.read_zeta3().unwrap(),
        |x| len_zeta(x, 3),
    );
    for k in 2..4 {
        bench!(
            format!("zeta_{}", k), 
            |w, x| w.write_zeta(x, k).unwrap(),
            |r| r.read_zeta(k).unwrap(),
            |x| len_zeta(x, k),
        );
    }
    for k in 2..5 {
        bench!(
            format!("pi_{}", k), 
            |w, x| w.write_pi(x, k).unwrap(),
            |r| r.read_pi(k).unwrap(),
            |x| len_pi(x, k),
        );
        bench!(
            format!("rice_{}", k), 
            |w, x| w.write_rice(x, k).unwrap(),
            |r| r.read_rice(k).unwrap(),
            |x| len_rice(x, k),
        );
        bench!(
            format!("exp_golomb_{}", k), 
            |w, x| w.write_exp_golomb(x, k).unwrap(),
            |r| r.read_exp_golomb(k).unwrap(),
            |x| len_exp_golomb(x, k),
        );
        bench!(
            format!("golomb_{}", k), 
            |w, x| w.write_exp_golomb(x, k).unwrap(),
            |r| r.read_exp_golomb(k).unwrap(),
            |x| len_exp_golomb(x, k),
        );
    }
}
