/*
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use common_traits::{DoubleType, UnsignedInt};
use dsi_bitstream::prelude::{
    BitRead, BitWrite, BufBitReader, BufBitWriter, MemWordReader, MemWordWriterVec,
};
use dsi_bitstream::traits::{Endianness, BE, LE};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
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

fn verify<E: Endianness, W: UnsignedInt + DoubleType, A: AsRef<[W]>>(
    buffer: A,
    mut len: u64,
    skip: usize,
    skip_read: bool,
) -> Result<(), Box<dyn std::error::Error>>
where
    BufBitReader<E, MemWordReader<W, A>>: BitRead<E>,
{
    let mut read = BufBitReader::<E, _>::new(MemWordReader::new(buffer));
    let mut r = SmallRng::seed_from_u64(0);
    if skip_read {
        len -= skip as u64;
        for _ in 0..skip {
            r.gen_range(0..2);
        }
    } else {
        read.skip_bits(skip)?;
    }

    for b in 0..len {
        assert_eq!(read.read_bits(1)?, r.gen_range(0..2), "@ {b}/{len}");
    }
    Ok(())
}

const MAX_LEN: u64 = 500;

fn test_endianness<'a, E: Endianness, W: UnsignedInt + DoubleType + 'static>(
) -> Result<(), Box<dyn std::error::Error>>
where
    BufBitReader<E, MemWordReader<W, Vec<W>>>: BitRead<E>,
    BufBitWriter<E, MemWordWriterVec<W, Vec<W>>>: BitWrite<E>,
{
    let mut write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<W>::new()));
    let mut r = SmallRng::seed_from_u64(0);
    for _ in 0..MAX_LEN {
        write.write_bits(r.gen_range(0..2), 1)?;
    }
    let buffer = write.into_inner()?.into_inner();

    for len in 0..MAX_LEN {
        // copy_to, BufBitReader implementation

        for skip in 0..=W::BITS.min(len as usize) {
            let mut read = BufBitReader::<E, _>::new(MemWordReader::new(buffer.clone()));
            let mut copy_write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<W>::new()));
            read.skip_bits(skip)?;
            read.copy_to(&mut copy_write, len - skip as u64)?;
            verify(copy_write.into_inner()?.into_inner(), len, skip, true)?;
        }

        // copy_from, BufBitWriter implementation

        for skip in 0..=W::BITS.min(len as usize) {
            let mut read = BufBitReader::<E, _>::new(MemWordReader::new(buffer.clone()));
            let mut copy_write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<W>::new()));
            for _ in 0..skip {
                copy_write.write_bits(0, 1)?;
            }
            copy_write.copy_from(&mut read, len)?;
            verify(copy_write.into_inner()?.into_inner(), len, skip, false)?;
        }
    }

    Ok(())
}
