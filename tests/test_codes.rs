/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use core::fmt;

use dsi_bitstream::dispatch::{
    CodeLen, CodesRead, CodesWrite, FuncCodeReader, FuncCodeWriter, MinimalBinary,
};
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
    test_func(
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
        test_codes_and_vals(code, &vals)?;
    }

    for k in 1..3 {
        test_codes_and_vals(Codes::ExpGolomb { k }, &[u64::MAX])?;
    }

    test_codes_and_vals(Codes::VByteBe, &[u64::MAX])?;
    test_codes_and_vals(Codes::VByteLe, &[u64::MAX])?;

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
        test_codes_and_vals(code, &vals)?;
    }

    test_func(
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

fn test_codes_and_vals<E: Endianness>(
    code: Codes,
    vals: &[u64],
) -> Result<(), Box<dyn std::error::Error>>
where
    for<'a> BW<'a, E>: CodesWrite<E>,
    for<'a> BR<'a, E>: CodesRead<E>,
{
    dbg!(code);
    let write_dispatch = FuncCodeWriter::new(code)?;
    let read_dispatch = FuncCodeReader::new(code)?;

    let mut buffer_call = Vec::<u64>::new();
    let mut buffer_disp = Vec::<u64>::new();
    let mut write_call = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer_call));
    let mut write_disp = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer_disp));

    for &val in vals {
        let len = code.write(&mut write_call, val)?;
        assert_eq!(len, code.len(val), "Code Write Len: {:?}", code);
        let len = write_dispatch.write(&mut write_disp, val)?;
        assert_eq!(len, code.len(val), "Dispatch Code Write Len: {:?}", code);
    }

    drop(write_call);
    drop(write_disp);
    let slice_call = unsafe { buffer_call.as_slice().align_to::<u32>().1 };
    let slice_disp = unsafe { buffer_disp.as_slice().align_to::<u32>().1 };

    let mut read_call = BufBitReader::<E, _>::new(MemWordReader::new(slice_call));
    let mut read_disp = BufBitReader::<E, _>::new(MemWordReader::new(slice_disp));

    for &val in vals {
        let value = code.read(&mut read_call)?;
        assert_eq!(value, val, "Code Read: {:?}", code);
        let value = read_dispatch.read(&mut read_disp)?;
        assert_eq!(value, val, "Dispatch Code Read: {:?}", code);
    }

    Ok(())
}

fn test_func<E: Endianness>(
    val_codes: &mut [(
        u64,
        impl DynamicCodeRead + DynamicCodeWrite + CodeLen + fmt::Debug,
    )],
) -> Result<(), Box<dyn std::error::Error>>
where
    for<'a> BW<'a, E>: CodesWrite<E>,
    for<'a> BR<'a, E>: CodesRead<E>,
{
    let mut buffer = Vec::<u64>::new();
    let mut write_call = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer));

    for (val, code) in val_codes.iter_mut() {
        let len = code.write(&mut write_call, *val)?;
        assert_eq!(len, code.len(*val));
    }

    drop(write_call);
    let slice_call = unsafe { buffer.as_slice().align_to::<u32>().1 };

    let mut read_call = BufBitReader::<E, _>::new(MemWordReader::new(slice_call));

    for (val, code) in val_codes.iter_mut() {
        let value = code.read(&mut read_call)?;
        assert_eq!(*val, value);
    }

    Ok(())
}
