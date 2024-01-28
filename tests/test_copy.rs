/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use dsi_bitstream::prelude::{
    BitRead, BitWrite, BufBitReader, BufBitWriter, MemWordReader, MemWordWriterVec,
};
use dsi_bitstream::traits::{Endianness, BE, LE};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    test_endianness::<LE>()?;
    test_endianness::<BE>()?;
    Ok(())
}

fn test_endianness<E: Endianness>() -> Result<(), Box<dyn std::error::Error>>
where
    BufBitWriter<E, MemWordWriterVec<u64, Vec<u64>>>: BitWrite<E>,
    BufBitReader<E, MemWordReader<u64, Vec<u64>>>: BitRead<E>,
{
    for len in 0..1000 {
        let mut write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<u64>::new()));

        let mut r = SmallRng::seed_from_u64(0);
        for _ in 0..len {
            write.write_bits(r.gen_range(0..1), 1)?;
        }

        let buffer = write.into_inner()?.into_inner();

        let mut read = BufBitReader::<E, _>::new(MemWordReader::new(buffer));

        let mut copy_write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<u64>::new()));

        read.copy_to(&mut copy_write, len)?;

        let mut read =
            BufBitReader::<E, _>::new(MemWordReader::new(copy_write.into_inner()?.into_inner()));

        let mut r = SmallRng::seed_from_u64(0);
        for _ in 0..len {
            assert_eq!(read.read_bits(1)?, r.gen_range(0..1));
        }

        let mut copy_write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<u64>::new()));
        copy_write.copy_from(&mut read, len)?;

        let mut read =
            BufBitReader::<E, _>::new(MemWordReader::new(copy_write.into_inner()?.into_inner()));

        let mut r = SmallRng::seed_from_u64(0);
        for _ in 0..len {
            assert_eq!(read.read_bits(1)?, r.gen_range(0..1));
        }
    }

    Ok(())
}
