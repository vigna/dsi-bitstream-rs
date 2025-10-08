use dsi_bitstream::prelude::*;
use dsi_bitstream::utils::sample_implied_distribution;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;

pub mod metrics;
use metrics::*;

pub mod utils;
use utils::*;

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
    univ: bool,
) where
    for<'a> BufBitWriter<E, MemWordWriterVec<WriteWord, &'a mut Vec<WriteWord>>>: BitWrite<E>,
    for<'a> BufBitReader<E, MemWordReader<ReadWord, &'a [ReadWord]>>: BitRead<E>,
{
    let mut rng = SmallRng::seed_from_u64(42);
    let samples_implied = sample_implied_distribution(&len, &mut rng)
        .take(N)
        .collect::<Vec<_>>();

    let samples_univ = if univ {
        let distr = rand_distr::Zipf::new(1E9 as f64, 1.0).unwrap();
        rng.sample_iter(distr)
            .map(|x| x as u64 - 1)
            .take(N)
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    // calculate the total number of bits we will write
    // so we can guarantee that the buffer always has enough capacity
    let total_bits = samples_implied
        .iter()
        .map(|&x| len(x) as u64)
        .sum::<u64>()
        .max(samples_univ.iter().map(|&x| len(x) as u64).sum::<u64>());

    let mut buffer: Vec<WriteWord> = Vec::with_capacity(total_bits.div_ceil(64) as usize + 10);

    let datasets = if univ {
        vec![(samples_implied, "implied"), (samples_univ, "univ")]
    } else {
        vec![(samples_implied, "implied")]
    };

    for (data, label) in datasets {
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
            "{:<12}\twrite:{label}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
            code.as_ref(),
            E::NAME,
            write.avg / N as f64,
            write.std / N as f64,
            write.percentile_25 / N as f64,
            write.median / N as f64,
            write.percentile_75 / N as f64,
        );
        println!(
            "{:<12}\tread:{label}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
            code.as_ref(),
            E::NAME,
            read.avg / N as f64,
            read.std / N as f64,
            read.percentile_25 / N as f64,
            read.median / N as f64,
            read.percentile_75 / N as f64,
        );
    }
}

/// Compares all codes using their implied distribution and a Zipf distribution
/// of exponent one.
pub fn main() {
    // tricks to reduce the noise
    #[cfg(target_os = "linux")]
    pin_to_core(5);

    // figure out how much overhead we add by measuring
    let calibration = calibrate_overhead();
    println!("{:<12}\trw\tendianness\tavg\tstd\t25%\tmedian\t75%", "code");

    macro_rules! bench {
        ($name:expr, $write:expr, $read:expr, $len:expr, $univ:expr) => {
            bench::<LE>(calibration, $name, $write, $read, $len, $univ);
            bench::<BE>(calibration, $name, $write, $read, $len, $univ);
        };
    }

    bench!(
        "unary",
        |w, x| w.write_unary(x).unwrap(),
        |r| r.read_unary().unwrap(),
        |x| x as usize + 1,
        false
    );
    bench!(
        "gamma",
        |w, x| w.write_gamma(x).unwrap(),
        |r| r.read_gamma().unwrap(),
        len_gamma,
        true
    );
    bench!(
        "delta",
        |w, x| w.write_delta(x).unwrap(),
        |r| r.read_delta().unwrap(),
        len_delta,
        true
    );
    bench!(
        "omega",
        |w, x| w.write_omega(x).unwrap(),
        |r| r.read_omega().unwrap(),
        len_omega,
        true
    );
    bench!(
        "vbyte_be",
        |w, x| w.write_vbyte_be(x).unwrap(),
        |r| r.read_vbyte_be().unwrap(),
        bit_len_vbyte,
        true
    );
    bench!(
        "vbyte_le",
        |w, x| w.write_vbyte_le(x).unwrap(),
        |r| r.read_vbyte_le().unwrap(),
        bit_len_vbyte,
        true
    );
    bench!(
        "zeta_3_table",
        |w, x| w.write_zeta3(x).unwrap(),
        |r| r.read_zeta3().unwrap(),
        |x| len_zeta(x, 3),
        true
    );
    for k in 2..4 {
        bench!(
            format!("zeta_{}", k),
            |w, x| w.write_zeta(x, k).unwrap(),
            |r| r.read_zeta(k).unwrap(),
            |x| len_zeta(x, k),
            true
        );
    }
    for k in 2..5 {
        bench!(
            format!("pi_{}", k),
            |w, x| w.write_pi(x, k).unwrap(),
            |r| r.read_pi(k).unwrap(),
            |x| len_pi(x, k),
            true
        );
        /*
        bench!(
            format!("pi_old_{}", k),
            |w, mut n| {
                n += 1; // π codes are indexed from 1
                let r = n.ilog2() as usize;
                let h = 1 + r;
                let l = h.div_ceil(1 << k);
                let v = (l * (1 << k) - h) as u64;
                let rem = n & !(u64::MAX << r);

                let mut written_bits = 0;
                written_bits += w.write_unary((l - 1) as u64).unwrap();
                written_bits += w.write_bits(v, k).unwrap();
                written_bits += w.write_bits(rem, r).unwrap();
                written_bits
            },
            |r| {
                let l = r.read_unary().unwrap() + 1;
                let v = r.read_bits(k).unwrap();
                let h = l * (1 << k) - v;
                let re = h - 1;
                let rem = r.read_bits(re as usize).unwrap();
                (1 << re) + rem - 1
            },
            |x| len_pi(x, k),
        );
         */
        bench!(
            format!("rice_{}", k),
            |w, x| w.write_rice(x, k).unwrap(),
            |r| r.read_rice(k).unwrap(),
            |x| len_rice(x, k),
            false
        );
        bench!(
            format!("exp_golomb_{}", k),
            |w, x| w.write_exp_golomb(x, k).unwrap(),
            |r| r.read_exp_golomb(k).unwrap(),
            |x| len_exp_golomb(x, k),
            true
        );
        bench!(
            format!("golomb_{}", k),
            |w, x| w.write_exp_golomb(x, k).unwrap(),
            |r| r.read_exp_golomb(k).unwrap(),
            |x| len_exp_golomb(x, k),
            false
        );
    }
}
