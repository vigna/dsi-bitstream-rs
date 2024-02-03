/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use dsi_bitstream::prelude::{
    BitRead, BitWrite, BufBitReader, BufBitWriter, DeltaRead, DeltaWrite, GammaRead, GammaWrite,
    GolombRead, GolombWrite, MemWordReader, MemWordWriterVec, MinimalBinaryRead,
    MinimalBinaryWrite, RiceRead, RiceWrite, ZetaRead, ZetaWrite,
};
use dsi_bitstream::traits::{Endianness, BE, LE};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::error::Error;

#[test]
fn test_codes() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    test_codes_endianness::<LE>()?;
    test_codes_endianness::<BE>()?;
    Ok(())
}

fn test_codes_endianness<E: Endianness>() -> Result<(), Box<dyn Error + Send + Sync + 'static>>
where
    BufBitWriter<E, MemWordWriterVec<u64, Vec<u64>>>:
        BitWrite<E> + GammaWrite<E> + DeltaWrite<E> + ZetaWrite<E> + GolombWrite<E> + RiceWrite<E>,
    BufBitReader<E, MemWordReader<u64, Vec<u64>>>:
        BitRead<E> + GammaRead<E> + DeltaRead<E> + ZetaRead<E> + GolombRead<E> + RiceRead<E>,
{
    const N: usize = 100000;
    let mut r = SmallRng::seed_from_u64(0);
    let mut v = SmallRng::seed_from_u64(1);
    let buffer = Vec::<u64>::new();
    let mut write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(buffer));

    let mut pos = vec![];

    for _ in 0..N {
        let mut written_bits = 0;
        match r.gen_range(0..8) {
            0 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_unary(v.gen_range(0..100))?;
                }
            }
            1 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_gamma(v.gen_range(0..100))?;
                }
            }
            2 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_delta(v.gen_range(0..100))?;
                }
            }
            3 => {
                let k = r.gen_range(2..4);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_zeta(v.gen_range(0..100), k)?;
                }
            }
            4 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_zeta3(v.gen_range(0..100))?;
                }
            }
            5 => {
                let max = r.gen_range(1..17);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_minimal_binary(v.gen_range(0..max), max)?;
                }
            }
            6 => {
                let b = r.gen_range(1..10);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_golomb(v.gen_range(0..100), b)?;
                }
            }
            7 => {
                let log2_b = r.gen_range(0..4);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_rice(v.gen_range(0..100), log2_b)?;
                }
            }
            _ => unreachable!(),
        }
        pos.push(written_bits);
    }

    let buffer = write.into_inner()?.into_inner();

    let mut read = BufBitReader::<E, _>::new(MemWordReader::new(buffer));

    let mut r = SmallRng::seed_from_u64(0);
    let mut v = SmallRng::seed_from_u64(1);

    for _ in 0..N {
        match r.gen_range(0..8) {
            0 => {
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_unary()?);
                }
            }
            1 => {
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_gamma()?);
                }
            }
            2 => {
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_delta()?);
                }
            }
            3 => {
                let k = r.gen_range(2..4);
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_zeta(k)?);
                }
            }
            4 => {
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_zeta3()?);
                }
            }
            5 => {
                let max = r.gen_range(1..17);
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..max), read.read_minimal_binary(max)?);
                }
            }
            6 => {
                let b = r.gen_range(1..10);
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_golomb(b)?);
                }
            }
            7 => {
                let log2_b = r.gen_range(0..4);
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_rice(log2_b)?);
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
