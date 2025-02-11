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
use criterion::{criterion_group, criterion_main, Criterion};
use dsi_bitstream::prelude::*;
use dsi_bitstream::traits::{BigEndian, BitRead, BitWrite, Endianness, LittleEndian};
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::hint::black_box;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::time::Duration;

pub const GAMMA_DATA: usize = 1_000_000;
pub const CAPACITY: usize = 4 * GAMMA_DATA;

pub fn gen_gamma_data(n: usize) -> Vec<u64> {
    let mut rng = SmallRng::seed_from_u64(0);
    let distr = rand_distr::Zipf::new(usize::MAX as f64, 8.0 / 7.0).unwrap();

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
    const INT: u8;
}
pub struct ZeroCont;
impl ContinuationBit for ZeroCont {
    const NAME: &'static str = "zero_cont";
    const INT: u8 = 0;
}
pub struct OneCont;
impl ContinuationBit for OneCont {
    const NAME: &'static str = "one_cont";
    const INT: u8 = 1;
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
    c.bench_function(&format!("vbyte,bytes,{},write", C::name()), |b| {
        b.iter(|| {
            let mut w = std::io::Cursor::new(&mut v);
            for &v in &s {
                black_box(C::write(v, &mut w).unwrap());
            }
        })
    });

    c.bench_function(&format!("vbyte,bytes,{},read", C::name()), |b| {
        b.iter(|| {
            let mut r = std::io::Cursor::new(v.as_slice());
            for _ in &s {
                black_box(C::read(&mut r).unwrap());
            }
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

    c.bench_function(
        &format!("vbyte,bits,{},{}_endian,write", C::name(), E::NAME),
        |b| {
            b.iter(|| {
                let mut w = <BufBitWriter<E, _>>::new(MemWordWriterVec::new(&mut v));
                for &v in &s {
                    black_box(C::write(v, &mut w).unwrap());
                }
            })
        },
    );

    let v = unsafe { v.align_to::<u32>().1 };

    c.bench_function(
        &format!("vbyte,bits,{},{}_endian,read", C::name(), E::NAME),
        |b| {
            b.iter(|| {
                let mut r = BufBitReader::<E, _>::new(MemWordReader::new(v));
                for _ in &s {
                    black_box(C::read(&mut r).unwrap());
                }
            })
        },
    );
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ByteStreamVByte<E: Endianness, F: Format, B: IsComplete, C: ContinuationBit>(
    PhantomData<(E, F, B, C)>,
);

impl<E: Endianness, F: Format, B: IsComplete, C: ContinuationBit> WithName
    for ByteStreamVByte<E, F, B, C>
{
    fn name() -> String {
        format!("{},{},{},{}_endian", F::NAME, B::NAME, C::NAME, E::NAME)
    }
}

impl<E: Endianness> ByteCode for ByteStreamVByte<E, GroupedIfs, Complete, OneCont> {
    fn read(r: &mut impl Read) -> Result<u64> {
        Ok(dsi_bitstream::codes::vbyte::vbyte_read::<E, _>(r)?)
    }
    fn write(value: u64, w: &mut impl Write) -> Result<usize> {
        Ok(dsi_bitstream::codes::vbyte::vbyte_write::<E, _>(value, w)?)
    }
}

const UPPER_BOUND_1: u64 = 128;
const UPPER_BOUND_2: u64 = 128_u64.pow(2);
const UPPER_BOUND_3: u64 = 128_u64.pow(3);
const UPPER_BOUND_4: u64 = 128_u64.pow(4);
const UPPER_BOUND_5: u64 = 128_u64.pow(5);
const UPPER_BOUND_6: u64 = 128_u64.pow(6);
const UPPER_BOUND_7: u64 = 128_u64.pow(7);
const UPPER_BOUND_8: u64 = 128_u64.pow(8);

impl ByteCode for ByteStreamVByte<BE, GroupedCLZ, NonComplete, OneCont> {
    fn read(r: &mut impl Read) -> Result<u64> {
        let mut buffer = [0; 8];
        r.read_exact(&mut buffer[..1])?;

        if buffer[0] == 0xFF {
            r.read_exact(&mut buffer)?;
            return Ok(u64::from_be_bytes(buffer).into());
        }

        let len = buffer[0].leading_ones() as usize;
        let result = buffer[0] as u64 & (0xFF >> len + 1);
        buffer[0] = 0;
        r.read_exact(&mut buffer[8 - len..])?;
        Ok(result << len * 8 | u64::from_be_bytes(buffer))
    }
    fn write(value: u64, w: &mut impl Write) -> Result<usize> {
        if value < UPPER_BOUND_1 {
            w.write_all(&[value as u8])?;
            return Ok(1);
        }
        if value < UPPER_BOUND_2 {
            debug_assert!((value >> 8) < (1 << 6));
            w.write_all(&[0x80 | (value >> 8) as u8, value as u8])?;
            return Ok(2);
        }
        if value < UPPER_BOUND_3 {
            debug_assert!((value >> 16) < (1 << 5));
            w.write_all(&[0xC0 | (value >> 16) as u8, (value >> 8) as u8, value as u8])?;
            return Ok(3);
        }
        if value < UPPER_BOUND_4 {
            debug_assert!((value >> 24) < (1 << 4));
            w.write_all(&[
                0xE0 | (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ])?;
            return Ok(4);
        }
        if value < UPPER_BOUND_5 {
            debug_assert!((value >> 32) < (1 << 3));
            w.write_all(&[
                0xF0 | (value >> 32) as u8,
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ])?;
            return Ok(5);
        }
        if value < UPPER_BOUND_6 {
            debug_assert!((value >> 40) < (1 << 2));
            w.write_all(&[
                0xF8 | (value >> 40) as u8,
                (value >> 32) as u8,
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ])?;
            return Ok(6);
        }
        if value < UPPER_BOUND_7 {
            debug_assert!((value >> 48) < (1 << 1));
            w.write_all(&[
                0xFC | (value >> 48) as u8,
                (value >> 40) as u8,
                (value >> 32) as u8,
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ])?;
            return Ok(7);
        }
        if value < UPPER_BOUND_8 {
            w.write_all(&[
                0xFE,
                (value >> 48) as u8,
                (value >> 40) as u8,
                (value >> 32) as u8,
                (value >> 24) as u8,
                (value >> 16) as u8,
                (value >> 8) as u8,
                value as u8,
            ])?;
            return Ok(8);
        }

        w.write_all(&[
            0xFF,
            (value >> 56) as u8,
            (value >> 48) as u8,
            (value >> 40) as u8,
            (value >> 32) as u8,
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ])?;
        Ok(9)
    }
}

/// LLVM's implementation https://llvm.org/doxygen/LEB128_8h_source.html#l00080
impl<B: IsComplete + 'static, C: ContinuationBit> ByteCode
    for ByteStreamVByte<LittleEndian, NonGrouped, B, C>
{
    fn read(r: &mut impl Read) -> Result<u64> {
        let mut result = 0;
        let mut shift = 0;
        let mut buffer = [0; 1];
        loop {
            r.read_exact(&mut buffer)?;
            let byte = buffer[0];
            result += ((byte & 0x7F) as u64) << shift;
            if (byte >> 7) == (1 - C::INT) {
                break;
            }
            shift += 7;
            if core::any::TypeId::of::<B>() == core::any::TypeId::of::<Complete>() {
                result += 1 << shift;
            }
        }
        Ok(result)
    }
    fn write(mut value: u64, w: &mut impl Write) -> Result<usize> {
        let mut len = 1;
        loop {
            let byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                w.write_all(&[byte | (0x80 * C::INT)])?;
            } else {
                w.write_all(&[byte | (0x80 * (1 - C::INT))])?;
                break;
            }
            if core::any::TypeId::of::<B>() == core::any::TypeId::of::<Complete>() {
                value -= 1;
            }
            len += 1;
        }
        Ok(len)
    }
}

/// Git implementation https://github.com/git/git/blob/7fb6aefd2aaffe66e614f7f7b83e5b7ab16d4806/varint.c#L4
impl<B: IsComplete + 'static, C: ContinuationBit> ByteCode
    for ByteStreamVByte<BigEndian, NonGrouped, B, C>
{
    fn read(r: &mut impl Read) -> Result<u64> {
        let mut buf = [0u8; 1];
        let mut value: u64;
        r.read_exact(&mut buf)?;
        value = (buf[0] & 0x7F) as u64;
        while (buf[0] >> 7) == C::INT {
            if core::any::TypeId::of::<B>() == core::any::TypeId::of::<Complete>() {
                value += 1;
            }
            r.read_exact(&mut buf)?;
            value = (value << 7) | ((buf[0] & 0x7F) as u64);
        }
        Ok(value)
    }

    fn write(mut value: u64, w: &mut impl Write) -> Result<usize> {
        let mut buf = [0u8; 10];
        let mut pos = buf.len() - 1;
        buf[pos] = (0x80 * (1 - C::INT)) | (value & 0x7F) as u8;
        value >>= 7;
        while value != 0 {
            if core::any::TypeId::of::<B>() == core::any::TypeId::of::<Complete>() {
                value -= 1;
            }
            pos -= 1;
            buf[pos] = (0x80 * C::INT) | ((value & 0x7F) as u8);
            value >>= 7;
        }
        let bytes_to_write = buf.len() - pos;
        w.write_all(&buf[pos..])?;
        Ok(bytes_to_write)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BitStreamVByte<F: Format, B: IsComplete, C: ContinuationBit>(PhantomData<(F, B, C)>);

impl<F: Format, B: IsComplete, C: ContinuationBit> WithName for BitStreamVByte<F, B, C> {
    fn name() -> String {
        format!("{},{},{}", F::NAME, B::NAME, C::NAME)
    }
}

impl BitCode for BitStreamVByte<GroupedIfs, Complete, OneCont> {
    #[inline(always)]
    fn read<E: Endianness>(r: &mut impl BitRead<E>) -> Result<u64> {
        Ok(r.read_vbyte_be()?)
    }
    #[inline(always)]
    fn write<E: Endianness>(value: u64, w: &mut impl BitWrite<E>) -> Result<usize> {
        Ok(w.write_vbyte_le(value)?)
    }
}

pub fn benchmark(c: &mut Criterion) {
    bench_bytestream::<ByteStreamVByte<LE, NonGrouped, Complete, OneCont>>(c);
    bench_bytestream::<ByteStreamVByte<LE, NonGrouped, Complete, ZeroCont>>(c);
    bench_bytestream::<ByteStreamVByte<LE, NonGrouped, NonComplete, OneCont>>(c);
    bench_bytestream::<ByteStreamVByte<LE, NonGrouped, NonComplete, ZeroCont>>(c);

    bench_bytestream::<ByteStreamVByte<BE, NonGrouped, Complete, OneCont>>(c);
    bench_bytestream::<ByteStreamVByte<BE, NonGrouped, Complete, ZeroCont>>(c);
    bench_bytestream::<ByteStreamVByte<BE, NonGrouped, NonComplete, OneCont>>(c);
    bench_bytestream::<ByteStreamVByte<BE, NonGrouped, NonComplete, ZeroCont>>(c);

    bench_bytestream::<ByteStreamVByte<LE, GroupedIfs, Complete, OneCont>>(c);
    bench_bytestream::<ByteStreamVByte<BE, GroupedIfs, Complete, OneCont>>(c);

    bench_bitstream::<BitStreamVByte<GroupedIfs, Complete, OneCont>>(c);

    //bench_bytestream::<ByteStreamVByte<BE, GroupedIfs, Complete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedIfs, NonComplete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedIfs, NonComplete, Zero>>(c);
    bench_bytestream::<ByteStreamVByte<BE, GroupedCLZ, NonComplete, OneCont>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedCLZ, Complete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedCLZ, NonComplete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<BE, GroupedCLZ, NonComplete, Zero>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, NonGrouped, Complete, One>>(c);
    //bench_bytestream::<ByteStreamVByte<LE, NonGrouped, Complete, Zero>>(c);
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
    //bench_bitstream::<BitStreamVByte<GroupedCLZ, NonComplete, Zero>>(c);
}

criterion_group! {
    name = vbyte_benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5))
        .sample_size(100);
    targets = benchmark
}
criterion_main!(vbyte_benches);
