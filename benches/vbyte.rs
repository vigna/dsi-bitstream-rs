//! Here we test 5 different choices:
//! - continuation bits at the start vs spread on each byte
//! - bits vs bytes streams
//! - little endian vs big endian
//! - 1 or 0 as continuation bit
//! - bijective or not
//!
//! - if continuation bits at start, test if chains vs leading_ones and jump table
//!
use anyhow::Result;
use common_traits::UnsignedInt;
use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::prelude::*;
use dsi_bitstream::traits::{BigEndian, BitRead, BitWrite, Endianness, LittleEndian};
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::hint::black_box;
use std::io::{Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::time::Duration;

pub const GAMMA_DATA: usize = 1 << 20;
pub const CAPACITY: usize = GAMMA_DATA;

pub fn gen_gamma_data(n: usize) -> Vec<u64> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distr = rand_distr::Zeta::new(2.0).unwrap();

    (0..n)
        .map(|_| rng.sample(distr) as u64 - 1)
        .collect::<Vec<_>>()
}

pub trait Format {
    const NAME: &'static str;
}
pub struct NonGrouped;
impl Format for NonGrouped {
    const NAME: &'static str = "non_grouped";
}
pub struct GroupedIfs;
impl Format for GroupedIfs {
    const NAME: &'static str = "grouped_ifs";
}
pub struct GroupedCLZ;
impl Format for GroupedCLZ {
    const NAME: &'static str = "grouped_clz";
}

pub trait ContinuationBit {
    const NAME: &'static str;
}
pub struct Zero;
impl ContinuationBit for Zero {
    const NAME: &'static str = "zero";
}
pub struct One;
impl ContinuationBit for One {
    const NAME: &'static str = "one";
}

pub trait IsComplete {
    const NAME: &'static str;
}
pub struct Complete;
impl IsComplete for Complete {
    const NAME: &'static str = "complete";
}
pub struct NonComplete;
impl IsComplete for NonComplete {
    const NAME: &'static str = "non_complete";
}

pub trait WithName {
    fn name() -> String;
}

pub trait ByteCode {
    fn read(r: &mut impl Read) -> Result<u64>;
    fn write(value: u64, w: &mut impl Write) -> Result<usize>;
}

pub fn bench_bytestream<C: ByteCode + WithName>(c: &mut Criterion) {
    let mut v = <Vec<u8>>::with_capacity(CAPACITY);
    // test that the impl works
    {
        let vals = (0..64)
            .map(|i| 1 << i)
            .chain(0..1024)
            .chain([u64::MAX])
            .collect::<Vec<_>>();
        let mut w = std::io::Cursor::new(&mut v);
        for v in &vals {
            C::write(*v, &mut w).unwrap();
        }
        let mut r = std::io::Cursor::new(v.as_slice());
        for v in &vals {
            assert_eq!(C::read(&mut r).unwrap(), *v);
        }
    }

    let s = gen_gamma_data(GAMMA_DATA);
    let mut i = 0;
    c.bench_function(&format!("vbyte,bytes,{},write", C::name()), |b| {
        let mut w = std::io::Cursor::new(&mut v);
        b.iter(|| {
            black_box(C::write(s[i % (GAMMA_DATA)], &mut w).unwrap());
            i += 1;
        })
    });

    c.bench_function(&format!("vbyte,bytes,{},read", C::name()), |b| {
        let mut r = std::io::Cursor::new(v.as_slice());
        b.iter(|| {
            black_box(C::read(&mut r).unwrap_or_else(|_| r.seek(SeekFrom::Start(0)).unwrap() as _));
        });
    });
}

pub trait BitCode {
    fn read<E: Endianness>(r: &mut impl BitRead<E>) -> Result<u64>;
    fn write<E: Endianness>(value: u64, w: &mut impl BitWrite<E>) -> Result<usize>;
}

pub fn bench_bitstream<C: BitCode + WithName>(c: &mut Criterion) {
    bench_bitstream_with_endiannes::<C, BigEndian>(c);
    bench_bitstream_with_endiannes::<C, LittleEndian>(c);
}

fn bench_bitstream_with_endiannes<C: BitCode + WithName, E: Endianness>(c: &mut Criterion)
where
    for<'a> BufBitReader<E, MemWordReader<u32, &'a [u32]>>: BitRead<E>,
    for<'a> BufBitWriter<E, MemWordWriterVec<u64, &'a mut Vec<u64>>>: BitWrite<E>,
{
    let mut v = <Vec<u64>>::with_capacity(CAPACITY);
    // test that the impl works
    {
        let vals = (0..64)
            .map(|i| 1 << i)
            .chain(0..1024)
            .chain([u64::MAX])
            .collect::<Vec<_>>();
        let mut w = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut v));
        for v in &vals {
            C::write(*v, &mut w).unwrap();
        }
        drop(w);
        let v = unsafe { v.align_to::<u32>().1 };
        let mut r = BufBitReader::<E, _>::new(MemWordReader::new(v));
        for v in &vals {
            assert_eq!(C::read(&mut r).unwrap(), *v);
        }
    }

    let s = gen_gamma_data(GAMMA_DATA);
    let mut i = 0;

    c.bench_function(
        &format!("vbyte,bits,{},{},write", C::name(), E::NAME),
        |b| {
            let mut w = <BufBitWriter<E, _>>::new(MemWordWriterVec::new(&mut v));
            b.iter(|| {
                black_box(C::write(s[i % (GAMMA_DATA)], &mut w).unwrap());
                i += 1;
            })
        },
    );

    let v = unsafe { v.align_to::<u32>().1 };

    c.bench_function(&format!("vbyte,bits,{},{},read", C::name(), E::NAME), |b| {
        let mut r = BufBitReader::<E, _>::new(MemWordReader::new(v));
        b.iter(|| {
            black_box(C::read(&mut r).unwrap_or_else(|_| {
                r = BufBitReader::<E, _>::new(MemWordReader::new(v));
                0
            }));
        })
    });
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ByteStreamVByte<E: Endianness, F: Format, B: IsComplete, C: ContinuationBit>(
    PhantomData<(E, F, B, C)>,
);

impl<E: Endianness, F: Format, B: IsComplete, C: ContinuationBit> WithName
    for ByteStreamVByte<E, F, B, C>
{
    fn name() -> String {
        format!("{},{},{},{}", F::NAME, B::NAME, C::NAME, E::NAME)
    }
}

impl<E: Endianness> ByteCode for ByteStreamVByte<E, GroupedIfs, Complete, One> {
    fn read(r: &mut impl Read) -> Result<u64> {
        Ok(dsi_bitstream::codes::vbyte::vbyte_decode::<E, _>(r)?)
    }
    fn write(value: u64, w: &mut impl Write) -> Result<usize> {
        Ok(dsi_bitstream::codes::vbyte::vbyte_encode::<E, _>(value, w)?)
    }
}

/// LLVM's implementation https://llvm.org/doxygen/LEB128_8h_source.html#l00080
impl ByteCode for ByteStreamVByte<LittleEndian, NonGrouped, NonComplete, One> {
    fn read(r: &mut impl Read) -> Result<u64> {
        let mut result = 0;
        let mut shift = 0;
        let mut buffer = [0; 1];
        loop {
            r.read_exact(&mut buffer)?;
            let byte = buffer[0];
            result |= ((byte & 0x7F) as u64) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        Ok(result)
    }
    fn write(mut value: u64, w: &mut impl Write) -> Result<usize> {
        let mut len = 1;
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                w.write_all(&[byte | 0x80])?;
            } else {
                w.write_all(&[byte])?;
                break;
            }
            len += 1;
        }
        Ok(len)
    }
}

/// Git implementation https://github.com/git/git/blob/7fb6aefd2aaffe66e614f7f7b83e5b7ab16d4806/varint.c#L4
impl ByteCode for ByteStreamVByte<BigEndian, NonGrouped, Complete, One> {
    fn read(r: &mut impl Read) -> Result<u64> {
        let mut result = 0;
        let mut buffer = [0; 1];
        loop {
            r.read_exact(&mut buffer)?;
            let byte = buffer[0];
            result = (result << 7) | ((byte & 0x7F) as u64);
            if (byte & 0x80) == 0 {
                break;
            }
        }
        Ok(result)
    }

    fn write(mut value: u64, w: &mut impl Write) -> Result<usize> {
        let mut pos = 8;
        let mut buffer = [0; 9];
        let mut byte = (value & 0x7F) as u8;
        buffer[pos] = byte;
        while byte != 0 {
            value >>= 7;
            byte = (value & 0x7F) as u8;
            buffer[pos - 1] = byte | 0x80;
            pos -= 1;
        }
        w.write_all(&buffer[pos..])?;
        Ok(9 - pos)
    }
}

impl ByteCode for ByteStreamVByte<BigEndian, NonGrouped, NonComplete, One> {
    fn read(r: &mut impl Read) -> Result<u64> {
        let mut result = 0;
        let mut buffer = [0; 1];
        loop {
            r.read_exact(&mut buffer)?;
            let byte = buffer[0];
            result = (result << 7) | ((byte & 0x7F) as u64);
            if (byte & 0x80) == 0 {
                break;
            }
        }
        Ok(result)
    }

    fn write(mut value: u64, w: &mut impl Write) -> Result<usize> {
        todo!();
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BitStreamVByte<F: Format, B: IsComplete, C: ContinuationBit>(PhantomData<(F, B, C)>);

impl<F: Format, B: IsComplete, C: ContinuationBit> WithName for BitStreamVByte<F, B, C> {
    fn name() -> String {
        format!("{},{},{}", F::NAME, B::NAME, C::NAME)
    }
}

impl BitCode for BitStreamVByte<GroupedIfs, Complete, One> {
    #[inline(always)]
    fn read<E: Endianness>(r: &mut impl BitRead<E>) -> Result<u64> {
        Ok(r.read_vbyte()?)
    }
    #[inline(always)]
    fn write<E: Endianness>(value: u64, w: &mut impl BitWrite<E>) -> Result<usize> {
        Ok(w.write_vbyte(value)?)
    }
}

impl BitCode for BitStreamVByte<GroupedCLZ, NonComplete, Zero> {
    #[inline(always)]
    fn read<E: Endianness>(r: &mut impl BitRead<E>) -> Result<u64> {
        let len = r.read_unary()? as usize;
        Ok(r.read_bits(len * 7)?)
    }
    #[inline(always)]
    fn write<E: Endianness>(value: u64, w: &mut impl BitWrite<E>) -> Result<usize> {
        let len = value.ilog2_ceil().div_ceil(7) as usize;
        w.write_unary(len as u64)?;
        w.write_bits(value, len * 7)?;
        Ok(len + 1)
    }
}

pub fn benchmark(c: &mut Criterion) {
    //bench_bytestream::<ByteStreamVByte<BE, NonGrouped, Complete, One>>(c);
    bench_bytestream::<ByteStreamVByte<LE, NonGrouped, NonComplete, One>>(c);
    bench_bytestream::<ByteStreamVByte<LE, GroupedIfs, Complete, One>>(c);
    bench_bytestream::<ByteStreamVByte<BE, GroupedIfs, Complete, One>>(c);
    bench_bitstream::<BitStreamVByte<GroupedIfs, Complete, One>>(c);
    //bench_bitstream::<BitStreamVByte<GroupedCLZ, NonComplete, Zero>>(c);

    //bench_bytestream::<ByteStreamVByte<BE, NonGrouped, Complete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, NonGrouped, NonComplete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, NonGrouped, NonComplete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedIfs, Complete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedIfs, NonComplete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedIfs, NonComplete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedCLZ, Complete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedCLZ, Complete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedCLZ, NonComplete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedCLZ, NonComplete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, NonGrouped, Complete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, NonGrouped, Complete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, NonGrouped, NonComplete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, GroupedIfs, Complete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, GroupedIfs, NonComplete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, GroupedIfs, NonComplete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, GroupedCLZ, Complete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, GroupedCLZ, Complete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, GroupedCLZ, NonComplete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, GroupedCLZ, NonComplete, Zero>>(c);
    //bench_bitstream::<BitStreamVByte<NonGrouped, Complete, One>>(c);
    //bench_bitstream::<BitStreamVByte<NonGrouped, Complete, Zero>>(c);
    //bench_bitstream::<BitStreamVByte<NonGrouped, NonComplete, One>>(c);
    //bench_bitstream::<BitStreamVByte<NonGrouped, NonComplete, Zero>>(c);
    //bench_bitstream::<BitStreamVByte<GroupedIfs, Complete, Zero>>(c);
    //bench_bitstream::<BitStreamVByte<GroupedIfs, NonComplete, One>>(c);
    //bench_bitstream::<BitStreamVByte<GroupedIfs, NonComplete, Zero>>(c);
    //bench_bitstream::<BitStreamVByte<GroupedCLZ, Complete, One>>(c);
    //bench_bitstream::<BitStreamVByte<GroupedCLZ, Complete, Zero>>(c);
    //bench_bitstream::<BitStreamVByte<GroupedCLZ, NonComplete, One>>(c);
}

criterion_group! {
    name = vbyte_benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = benchmark
}
criterion_main!(vbyte_benches);
