/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */
#![cfg(feature = "alloc")]

use core::fmt;

use dsi_bitstream::dispatch::{CodeLen, CodesRead, CodesWrite, MinimalBinary};
use dsi_bitstream::prelude::*;

type BW<'a, E> = BufBitWriter<E, MemWordWriterVec<u64, &'a mut Vec<u64>>>;
type BR<'a, E> = BufBitReader<E, MemWordReader<u32, &'a [u32]>>;

#[test]
fn test_codes() -> Result<(), Box<dyn std::error::Error>> {
    test_codes_endianness::<LE>()?;
    test_codes_endianness::<BE>()?;
    Ok(())
}

fn test_codes_endianness<E: Endianness>() -> Result<(), Box<dyn std::error::Error>>
where
    for<'a> BW<'a, E>: CodesWrite<E>,
    for<'a> BR<'a, E>: CodesRead<E>,
{
    test_vals_codes(
        [
            (0, MinimalBinary(1)),
            (0, MinimalBinary(2)),
            (1, MinimalBinary(2)),
            (0, MinimalBinary(3)),
            (1, MinimalBinary(3)),
            (2, MinimalBinary(3)),
            (0, MinimalBinary(u64::MAX)),
            (1, MinimalBinary(u64::MAX)),
            (u64::MAX / 2 - 1, MinimalBinary(u64::MAX)),
            (u64::MAX / 2, MinimalBinary(u64::MAX)),
            (u64::MAX / 2 + 1, MinimalBinary(u64::MAX)),
            (u64::MAX - 1, MinimalBinary(u64::MAX)),
        ]
        .as_mut_slice(),
    )?;

    // Codes that handle u64::MAX - 1
    let mut codes = vec![
        Codes::Gamma,
        Codes::Delta,
        Codes::Omega,
        Codes::VByteBe,
        Codes::VByteLe,
    ];
    for i in 0..10_usize {
        codes.push(Codes::Pi { k: i });
        codes.push(Codes::ExpGolomb { k: i });
    }
    for i in 1..10_usize {
        codes.push(Codes::Zeta { k: i });
    }
    let vals = (0..64)
        .map(|i| 1 << i)
        .chain(0..1024)
        .chain([u64::MAX - 1])
        .collect::<Vec<_>>();
    for code in codes {
        test_code_with_vals(code, &vals)?;
    }

    for k in 1..3 {
        test_code_with_vals(Codes::ExpGolomb { k }, &[u64::MAX])?;
    }

    test_code_with_vals(Codes::VByteBe, &[u64::MAX])?;
    test_code_with_vals(Codes::VByteLe, &[u64::MAX])?;

    // codes that would generate code words too big to be handled for
    // u64::MAX - 1
    let mut sparse_codes = vec![Codes::Unary];
    for i in 0..10 {
        sparse_codes.push(Codes::Rice { log2_b: i });
    }
    for i in 1..10 {
        sparse_codes.push(Codes::Golomb { b: i });
    }
    let vals = (0..1024).collect::<Vec<_>>();
    for code in sparse_codes {
        test_code_with_vals(code, &vals)?;
    }

    test_vals_codes(
        [
            (0, Codes::Golomb { b: u64::MAX }),
            (u64::MAX / 2, Codes::Golomb { b: u64::MAX }),
            (u64::MAX, Codes::Golomb { b: u64::MAX }),
            (0, Codes::Rice { log2_b: 63 }),
            (u64::MAX / 2, Codes::Rice { log2_b: 63 }),
            (u64::MAX, Codes::Rice { log2_b: 63 }),
        ]
        .as_mut_slice(),
    )?;

    Ok(())
}

fn test_code_with_vals<E: Endianness>(
    code: Codes,
    vals: &[u64],
) -> Result<(), Box<dyn std::error::Error>>
where
    for<'a> BW<'a, E>: CodesWrite<E>,
    for<'a> BR<'a, E>: CodesRead<E>,
{
    dbg!(code);

    let mut v = vals.iter().map(|&v| (v, code)).collect::<Vec<_>>();
    test_vals_codes(v.as_mut_slice())
}

fn test_vals_codes<E: Endianness>(
    val_codes: &mut [(
        u64,
        impl DynamicCodeRead
        + DynamicCodeWrite
        + for<'a> StaticCodeRead<E, BR<'a, E>>
        + for<'b> StaticCodeWrite<E, BW<'b, E>>
        + CodeLen
        + fmt::Debug,
    )],
) -> Result<(), Box<dyn std::error::Error>>
where
    for<'a> BW<'a, E>: CodesWrite<E>,
    for<'a> BR<'a, E>: CodesRead<E>,
{
    let mut buffer_dynamic = Vec::<u64>::new();
    let mut buffer_static = Vec::<u64>::new();
    let mut write_dynamic = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer_dynamic));
    let mut write_static = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer_static));

    for (val, code) in val_codes.iter_mut() {
        let len = DynamicCodeWrite::write(code, &mut write_dynamic, *val)?;
        assert_eq!(len, code.len(*val));
        let len = StaticCodeWrite::write(code, &mut write_static, *val)?;
        assert_eq!(len, code.len(*val));
    }

    drop(write_dynamic);
    drop(write_static);
    let slice_dynamic = unsafe { buffer_dynamic.as_slice().align_to::<u32>().1 };
    let slice_static = unsafe { buffer_static.as_slice().align_to::<u32>().1 };

    let mut read_dynamic = BufBitReader::<E, _>::new(MemWordReader::new(slice_dynamic));
    let mut read_static = BufBitReader::<E, _>::new(MemWordReader::new(slice_static));

    for (val, code) in val_codes.iter_mut() {
        let value = DynamicCodeRead::read(code, &mut read_dynamic)?;
        assert_eq!(*val, value);
        let value = StaticCodeRead::read(code, &mut read_static)?;
        assert_eq!(*val, value);
    }

    Ok(())
}
