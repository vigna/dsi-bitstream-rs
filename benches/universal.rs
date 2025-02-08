use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::codes::{OmegaRead, OmegaWrite};
use dsi_bitstream::impls::{BufBitReader, BufBitWriter, MemWordReader, MemWordWriterVec};
use dsi_bitstream::traits::{BitRead, BitWrite, Endianness, BE, LE};
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::hint::black_box;
use std::time::Duration;
const GAMMA_DATA: usize = 1000000;
const CAPACITY: usize = GAMMA_DATA * 4;

// Levenshtein's universal code.

#[inline(always)]
pub fn len_levenshtein(n: u64) -> usize {
    if n == 0 {
        return 1;
    }
    recursive_len(1, n)
}

fn recursive_len(blocks: usize, n: u64) -> usize {
    if n == 1 {
        return blocks + 1;
    }
    let λ = n.ilog2();
    recursive_len(blocks + 1, λ as u64) + λ as usize
}

pub trait LevenshteinRead<E: Endianness>: BitRead<E> {
    // Levenshtein codes are indexed from 1
    fn read_levenshtein(&mut self) -> Result<u64, Self::Error> {
        let λ = self.read_unary()?;
        if λ == 0 {
            return Ok(0);
        }
        let mut block_len = 0_u64;
        for _ in 0..λ {
            let block = self.read_bits(block_len as usize)?;
            block_len = (1 << block_len) | block;
        }

        Ok(block_len)
    }
}

pub trait LevenshteinWrite<E: Endianness>: BitWrite<E> {
    fn write_levenshtein(&mut self, n: u64) -> Result<usize, Self::Error> {
        if n == 0 {
            return self.write_bits(1, 1);
        }
        recursive_write::<E, Self>(self, 1, n)
    }
}

fn recursive_write<E: Endianness, B: BitWrite<E> + ?Sized>(
    writer: &mut B,
    blocks: usize,
    n: u64,
) -> Result<usize, B::Error> {
    if n == 1 {
        return writer.write_unary(blocks as u64);
    }
    let λ = n.ilog2() as usize;
    Ok(recursive_write(writer, blocks + 1, λ as u64)? + writer.write_bits(n, λ)?)
}

impl<E: Endianness, B: BitRead<E>> LevenshteinRead<E> for B {}
impl<E: Endianness, B: BitWrite<E>> LevenshteinWrite<E> for B {}

#[cfg(test)]
mod test {
    #[test]
    fn test_roundtrip() {
        for value in (0..64).map(|i| 1 << i).chain(0..1024).chain([u64::MAX]) {
            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
            let code_len = writer.write_levenshtein(value).unwrap();
            assert_eq!(code_len, len_levenshtein(value));
            drop(writer);
            let mut reader = <BufBitReader<BE, _>>::new(MemWordReader::new(&data));
            assert_eq!(reader.read_levenshtein().unwrap(), value);

            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(&mut data));
            let code_len = writer.write_levenshtein(value).unwrap();
            assert_eq!(code_len, len_levenshtein(value));
            drop(writer);
            let mut reader = <BufBitReader<LE, _>>::new(MemWordReader::new(&data));
            assert_eq!(reader.read_levenshtein().unwrap(), value,);
        }
    }

    #[test]
    fn test_bits() {
        for (value, len, expected_be, expected_le) in [
            (0, 1, 1 << 63, 1),
            (1, 2, 0b01 << (64 - 2), 0b10),
            (2, 4, 0b001_0 << (64 - 4), 0b0_100),
            (3, 4, 0b001_1 << (64 - 4), 0b1_100),
            (4, 7, 0b0001_0_00 << (64 - 7), 0b_00_0_1000),
            (5, 7, 0b0001_0_01 << (64 - 7), 0b_01_0_1000),
            (6, 7, 0b0001_0_10 << (64 - 7), 0b_10_0_1000),
            (7, 7, 0b0001_0_11 << (64 - 7), 0b_11_0_1000),
            (15, 8, 0b0001_1_111 << (64 - 8), 0b111_1_1000),
            (
                99,
                14,
                0b00001_0_10_100011 << (64 - 14),
                0b100011_10_0_10000,
            ),
            (
                999,
                18,
                0b00001_1_001_111100111 << (64 - 18),
                0b111100111_001_1_10000,
            ),
            (
                999_999,
                32,
                0b000001_0_00_0011_1110100001000111111 << (64 - 32),
                0b1110100001000111111_0011_00_0_100000,
            ),
        ] {
            assert_eq!(len_levenshtein(value), len);

            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
            assert_eq!(writer.write_levenshtein(value).unwrap(), len);

            drop(writer);
            assert_eq!(
                data[0].to_be(),
                expected_be,
                "\nfor value: {}\ngot: {:064b}\nexp: {:064b}\n",
                value,
                data[0].to_be(),
                expected_be,
            );

            let mut data = vec![0_u64];
            let mut writer = <BufBitWriter<LE, _>>::new(MemWordWriterVec::new(&mut data));
            assert_eq!(writer.write_levenshtein(value).unwrap(), len);
            drop(writer);
            assert_eq!(
                data[0].to_le(),
                expected_le,
                "\nfor value: {}\ngot: {:064b}\nexp: {:064b}\n",
                value,
                data[0].to_le(),
                expected_le,
            );
        }
    }
}

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
