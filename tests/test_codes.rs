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
        match r.gen_range(0..11) {
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
                let log2_b = r.gen_range(1..5);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_rice(v.gen_range(0..100), log2_b)?;
                }
            }
            8 => {
                let k = r.gen_range(1..5);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_exp_golomb(v.gen_range(0..100), k)?;
                }
            }
            9 => {
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_omega(v.gen_range(0..100))?;
                }
            }
            10 => {
                let k = r.gen_range(1..4);
                for _ in 0..r.gen_range(1..10) {
                    written_bits += write.write_pi(v.gen_range(0..100), k)?;
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
        match r.gen_range(0..11) {
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
                let log2_b = r.gen_range(1..5);
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_rice(log2_b)?);
                }
            }
            8 => {
                let k = r.gen_range(1..5);
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_exp_golomb(k)?);
                }
            }
            9 => {
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_omega()?);
                }
            }
            10 => {
                let k = r.gen_range(1..4);
                for _ in 0..r.gen_range(1..10) {
                    assert_eq!(v.gen_range(0..100), read.read_pi(k)?);
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

#[test]
fn test_pi_roundtrip() {
    let k = 3;
    for value in 0..1_000_000 {
        let mut data = vec![0_u64];
        let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
        let code_len = writer.write_pi(value, k).unwrap();
        assert_eq!(code_len, len_pi(value, k));
        drop(writer);
        let mut reader = <BufBitReader<BE, _>>::new(MemWordReader::new(&data));
        assert_eq!(
            reader.read_pi(k).unwrap(),
            value,
            "for value: {} with k {}",
            value,
            k
        );
    }
}

#[test]
fn test_pi() {
    for (k, value, expected) in [
        (2, 20, 0b01_00_0101 << (64 - 8)),
        (2, 0, 0b100 << (64 - 3)),
        (2, 1, 0b1010 << (64 - 4)),
        (2, 2, 0b1011 << (64 - 4)),
        (2, 3, 0b1_1000 << (64 - 5)),
        (2, 4, 0b1_1001 << (64 - 5)),
        (2, 5, 0b1_1010 << (64 - 5)),
        (2, 6, 0b1_1011 << (64 - 5)),
        (2, 7, 0b11_1000 << (64 - 6)),
        (3, 0, 0b1000 << (64 - 4)),
        (3, 1, 0b1_0010 << (64 - 5)),
        (3, 2, 0b1_0011 << (64 - 5)),
        (3, 3, 0b1_01000 << (64 - 6)),
        (3, 4, 0b1_01001 << (64 - 6)),
        (3, 5, 0b1_01010 << (64 - 6)),
        (3, 6, 0b1_01011 << (64 - 6)),
        (3, 7, 0b101_1000 << (64 - 7)),
    ] {
        let mut data = vec![0_u64];
        let mut writer = <BufBitWriter<BE, _>>::new(MemWordWriterVec::new(&mut data));
        let code_len = writer.write_pi(value, k).unwrap();
        drop(writer);
        assert_eq!(
            data[0].to_be(),
            expected,
            "\nfor value: {} with k {}\ngot: {:064b}\nexp: {:064b}\ngot_len: {} exp_len: {}\n",
            value,
            k,
            data[0].to_be(),
            expected,
            code_len,
            len_pi(value, k),
        );
    }
}
