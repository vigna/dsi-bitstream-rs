/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![cfg(feature = "alloc")]
use core::error::Error;
use dsi_bitstream::prelude::{
    BitRead, BitWrite, BufBitReader, BufBitWriter, MemWordReader, MemWordWriterVec,
};
use dsi_bitstream::traits::{BE, DoubleType, Endianness, LE, Word};
use num_primitive::PrimitiveInteger;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

#[test]
fn test_copy() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    test_endianness::<LE, u8>()?;
    test_endianness::<BE, u8>()?;
    test_endianness::<LE, u16>()?;
    test_endianness::<BE, u16>()?;
    test_endianness::<LE, u32>()?;
    test_endianness::<BE, u32>()?;
    test_endianness::<LE, u64>()?;
    test_endianness::<BE, u64>()?;
    Ok(())
}

fn verify_read<E: Endianness>(
    mut read: impl BitRead<E>,
    len: u64,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let mut r = SmallRng::seed_from_u64(0);

    for _ in 0..len {
        let _: u64 = r.random_range(0..2);
    }

    for _ in len..MAX_LEN {
        assert_eq!(read.read_bits(1)?, r.random_range(0..2));
    }

    Ok(())
}

fn verify_write<E: Endianness, W: Word + DoubleType, A: AsRef<[W]>>(
    buffer: A,
    mut len: u64,
    skip: usize,
    skip_read: bool,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
    BufBitReader<E, MemWordReader<W, A, true>>: BitRead<E>,
{
    let mut read = BufBitReader::<E, _>::new(MemWordReader::new_inf(buffer));
    let mut r = SmallRng::seed_from_u64(0);
    if skip_read {
        len -= skip as u64;
        for _ in 0..skip {
            let _: u64 = r.random_range(0..2);
        }
    } else {
        read.skip_bits(skip)?;
    }

    for b in 0..len {
        assert_eq!(read.read_bits(1)?, r.random_range(0..2), "@ {b}/{len}");
    }

    let mut r = SmallRng::seed_from_u64(1);
    for _ in 0..100 {
        assert_eq!(read.read_bits(1)?, r.random_range(0..2));
    }

    Ok(())
}

const MAX_LEN: u64 = 500;

#[test]
fn test_copy_from_u128_words() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // Regression test: copy_from used to request more than 64 bits at a
    // time from read_bits when the writer word is u128
    let mut write = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(Vec::<u32>::new()));
    let mut r = SmallRng::seed_from_u64(0);
    for _ in 0..MAX_LEN {
        write.write_bits(r.random_range(0..2), 1)?;
    }
    let buffer = write.into_inner()?.into_inner();

    let mut read = BufBitReader::<BE, _>::new(MemWordReader::new_inf(buffer));
    let mut copy_write = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(Vec::<u128>::new()));
    copy_write.copy_from(&mut read, 100)?;
    let mut r1 = SmallRng::seed_from_u64(1);
    for _ in 0..100 {
        copy_write.write_bits(r1.random_range(0..2), 1)?;
    }
    let copy = copy_write.into_inner()?.into_inner();

    // Convert the u128 words to u32 words to read the copy back (u128 has
    // no double type, so it cannot be used as a read word)
    let bytes: Vec<u8> = copy.iter().flat_map(|w| w.to_ne_bytes()).collect();
    let words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|c| u32::from_ne_bytes(c.try_into().unwrap()))
        .collect();
    let mut check = BufBitReader::<BE, _>::new(MemWordReader::new_inf(words));
    let mut r = SmallRng::seed_from_u64(0);
    for _ in 0..100 {
        assert_eq!(check.read_bits(1)?, r.random_range(0..2));
    }
    let mut r1 = SmallRng::seed_from_u64(1);
    for _ in 0..100 {
        assert_eq!(check.read_bits(1)?, r1.random_range(0..2));
    }
    Ok(())
}

fn test_endianness<E: Endianness, W: Word + PrimitiveInteger + DoubleType + 'static>()
-> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
    BufBitReader<E, MemWordReader<W, Vec<W>, true>>: BitRead<E>,
    BufBitWriter<E, MemWordWriterVec<W, Vec<W>>>: BitWrite<E>,
{
    let mut write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<W>::new()));
    let mut r = SmallRng::seed_from_u64(0);
    for _ in 0..MAX_LEN {
        write.write_bits(r.random_range(0..2), 1)?;
    }
    let buffer = write.into_inner()?.into_inner();

    for len in 0..MAX_LEN {
        // copy_to, BufBitReader implementation

        for skip in 0..=(W::BITS as usize).min(len as usize) {
            let mut read = BufBitReader::<E, _>::new(MemWordReader::new_inf(buffer.clone()));
            let mut copy_write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<W>::new()));
            read.skip_bits(skip)?;
            read.copy_to(&mut copy_write, len - skip as u64)?;

            let mut r = SmallRng::seed_from_u64(1);
            for _ in 0..100 {
                copy_write.write_bits(r.random_range(0..2), 1)?;
            }

            verify_write(copy_write.into_inner()?.into_inner(), len, skip, true)?;
            verify_read(read, len)?;
        }

        // copy_from, BufBitWriter implementation

        for skip in 0..=(W::BITS as usize).min(len as usize) {
            let mut read = BufBitReader::<E, _>::new(MemWordReader::new_inf(buffer.clone()));
            let mut copy_write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<W>::new()));
            for _ in 0..skip {
                copy_write.write_bits(0, 1)?;
            }
            copy_write.copy_from(&mut read, len)?;

            let mut r = SmallRng::seed_from_u64(1);
            for _ in 0..100 {
                copy_write.write_bits(r.random_range(0..2), 1)?;
            }

            verify_write(copy_write.into_inner()?.into_inner(), len, skip, false)?;
            verify_read(read, len)?;
        }
    }

    Ok(())
}

/// 16 varied words; word 0 has both its MSB and LSB set so that a leading
/// 1-bit read is nonzero for either endianness.
fn topup_sample_words() -> [u64; 16] {
    [
        0x8000_0000_0000_0001,
        0xfedc_ba98_7654_3210,
        0x0123_4567_89ab_cdef,
        0xdead_beef_cafe_babe,
        0x1111_2222_3333_4444,
        0x5555_6666_7777_8888,
        0x9999_aaaa_bbbb_cccc,
        0xdddd_eeee_ffff_0000,
        0x0f0f_0f0f_0f0f_0f0f,
        0xf0f0_f0f0_f0f0_f0f0,
        0xaaaa_5555_aaaa_5555,
        0x1234_1234_1234_1234,
        0xabcd_ef01_2345_6789,
        0x9e37_79b9_7f4a_7c15,
        0x6a09_e667_f3bc_c908,
        0xbb67_ae85_84ca_a73b,
    ]
}

fn read_bit_seq<E: Endianness>(r: &mut impl BitRead<E>, n: usize) -> Vec<u64> {
    (0..n).map(|_| r.read_bits(1).unwrap()).collect()
}

/// After a 1-bit read, the two-word refill top-up leaves `2 * WORD_BITS - 1`
/// bits in the bit buffer (more than one word), so a bulk `copy_to` must
/// drain more than `WORD_BITS` buffered bits without exceeding the 64-bit
/// `write_bits` contract.
fn copy_to_after_topup<E: Endianness, W: Word + PrimitiveInteger + DoubleType + 'static>(
    words: &[W],
) where
    BufBitReader<E, MemWordReader<W, Vec<W>, true>>: BitRead<E>,
    BufBitReader<E, MemWordReader<u64, Vec<u64>, true>>: BitRead<E>,
    BufBitWriter<E, MemWordWriterVec<u64, Vec<u64>>>: BitWrite<E>,
{
    const N: usize = 200;
    let mut refr = BufBitReader::<E, _>::new(MemWordReader::new_inf(words.to_vec()));
    let _ = refr.read_bits(1).unwrap();
    let expected = read_bit_seq::<E>(&mut refr, N);

    let mut src = BufBitReader::<E, _>::new(MemWordReader::new_inf(words.to_vec()));
    let _ = src.read_bits(1).unwrap();
    let mut wr = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<u64>::new()));
    // Lossless: N is a small constant and usize is at most 64 bits wide.
    src.copy_to(&mut wr, N as u64).unwrap();
    let out = wr.into_inner().unwrap().into_inner();
    let mut back = BufBitReader::<E, _>::new(MemWordReader::new_inf(out));
    assert_eq!(read_bit_seq::<E>(&mut back, N), expected);
}

#[test]
fn test_copy_to_after_topup() {
    let w64 = topup_sample_words();
    copy_to_after_topup::<BE, u64>(&w64);
    copy_to_after_topup::<LE, u64>(&w64);
    let w32: Vec<u32> = w64
        .iter()
        // Intended truncation: split each u64 into its two u32 halves.
        .flat_map(|w| [(*w >> 32) as u32, *w as u32])
        .collect();
    copy_to_after_topup::<BE, u32>(&w32);
    copy_to_after_topup::<LE, u32>(&w32);
}
