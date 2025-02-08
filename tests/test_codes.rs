/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use dsi_bitstream::prelude::*;
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
    BufBitWriter<E, MemWordWriterVec<u64, Vec<u64>>>: BitWrite<E>
        + GammaWrite<E>
        + DeltaWrite<E>
        + ZetaWrite<E>
        + GolombWrite<E>
        + RiceWrite<E>
        + OmegaWrite<E>
        + PiWrite<E>,
    BufBitReader<E, MemWordReader<u64, Vec<u64>>>: BitRead<E>
        + GammaRead<E>
        + DeltaRead<E>
        + ZetaRead<E>
        + GolombRead<E>
        + RiceRead<E>
        + OmegaRead<E>
        + PiRead<E>,
{
    const N: usize = 100000;
    let mut r = SmallRng::seed_from_u64(0);
    let mut v = SmallRng::seed_from_u64(1);
    let buffer = Vec::<u64>::new();
    let mut write = BufBitWriter::<E, _>::new(MemWordWriterVec::new(buffer));

    let mut pos = vec![];

    for _ in 0..N {
        let mut written_bits = 0;
        match r.random_range(0..11) {
            0 => {
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_unary(v.random_range(0..100))?;
                }
            }
            1 => {
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_gamma(v.random_range(0..100))?;
                }
            }
            2 => {
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_delta(v.random_range(0..100))?;
                }
            }
            3 => {
                let k = r.random_range(2..4);
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_zeta(v.random_range(0..100), k)?;
                }
            }
            4 => {
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_zeta3(v.random_range(0..100))?;
                }
            }
            5 => {
                let max = r.random_range(1..17);
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_minimal_binary(v.random_range(0..max), max)?;
                }
            }
            6 => {
                let b = r.random_range(1..10);
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_golomb(v.random_range(0..100), b)?;
                }
            }
            7 => {
                let log2_b = r.random_range(1..5);
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_rice(v.random_range(0..100), log2_b)?;
                }
            }
            8 => {
                let k = r.random_range(1..5);
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_exp_golomb(v.random_range(0..100), k)?;
                }
            }
            9 => {
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_omega(v.random_range(0..100))?;
                }
            }
            10 => {
                let k = r.random_range(1..4);
                for _ in 0..r.random_range(1..10) {
                    written_bits += write.write_pi(v.random_range(0..100), k)?;
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
        match r.random_range(0..11) {
            0 => {
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_unary()?);
                }
            }
            1 => {
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_gamma()?);
                }
            }
            2 => {
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_delta()?);
                }
            }
            3 => {
                let k = r.random_range(2..4);
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_zeta(k)?);
                }
            }
            4 => {
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_zeta3()?);
                }
            }
            5 => {
                let max = r.random_range(1..17);
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..max), read.read_minimal_binary(max)?);
                }
            }
            6 => {
                let b = r.random_range(1..10);
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_golomb(b)?);
                }
            }
            7 => {
                let log2_b = r.random_range(1..5);
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_rice(log2_b)?);
                }
            }
            8 => {
                let k = r.random_range(1..5);
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_exp_golomb(k)?);
                }
            }
            9 => {
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_omega()?);
                }
            }
            10 => {
                let k = r.random_range(1..4);
                for _ in 0..r.random_range(1..10) {
                    assert_eq!(v.random_range(0..100), read.read_pi(k)?);
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
