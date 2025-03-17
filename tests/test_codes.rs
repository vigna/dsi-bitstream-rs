/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use dsi_bitstream::dispatch::{CodeLen, CodesRead, CodesWrite, FuncCodeReader, FuncCodeWriter};
use dsi_bitstream::prelude::*;

#[test]
fn test_codes() {
    test_codes_endianness::<LE>();
    test_codes_endianness::<BE>();
}

fn test_codes_endianness<E: Endianness>()
where
    for<'a> BufBitWriter<E, MemWordWriterVec<u64, &'a mut Vec<u64>>>: CodesWrite<E>,
    for<'a> BufBitReader<E, MemWordReader<u32, &'a [u32]>>: CodesRead<E>,
{
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
        test_codes_and_vals(code, &vals);
    }

    // codes that would generate code words too big to be handled for
    // u64::MAX - 1
    let mut sparse_codes = vec![Codes::Unary];
    for i in 0..10_usize {
        sparse_codes.push(Codes::Rice { log2_b: i });
    }
    for i in 1..10_usize {
        sparse_codes.push(Codes::Golomb { b: i });
    }
    let vals = (0..1024).collect::<Vec<_>>();
    for code in sparse_codes {
        test_codes_and_vals(code, &vals);
    }
}

fn test_codes_and_vals<E: Endianness>(code: Codes, vals: &[u64])
where
    for<'a> BufBitWriter<E, MemWordWriterVec<u64, &'a mut Vec<u64>>>: CodesWrite<E>,
    for<'a> BufBitReader<E, MemWordReader<u32, &'a [u32]>>: CodesRead<E>,
{
    dbg!(code);
    let write_dispatch = FuncCodeWriter::new(code).unwrap();
    let read_dispatch = FuncCodeReader::new(code).unwrap();

    let mut buffer1 = Vec::<u64>::new();
    let mut buffer2 = Vec::<u64>::new();
    let mut write1 = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer1));
    let mut write2 = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer2));

    for &val in vals {
        let len = code.write(&mut write1, val).unwrap();
        assert_eq!(len, code.len(val), "Code Write Len: {:?}", code);
        let len = write_dispatch.write(&mut write2, val).unwrap();
        assert_eq!(len, code.len(val), "Dispatch Code Write Len: {:?}", code);
    }

    drop(write1);
    drop(write2);
    let slice1 = unsafe { buffer1.as_slice().align_to::<u32>().1 };
    let slice2 = unsafe { buffer2.as_slice().align_to::<u32>().1 };

    let mut read1 = BufBitReader::<E, _>::new(MemWordReader::new(slice1));
    let mut read2 = BufBitReader::<E, _>::new(MemWordReader::new(slice2));

    for &val in vals {
        let value = code.read(&mut read1).unwrap();
        assert_eq!(value, val, "Code Read: {:?}", code);
        let value = read_dispatch.read(&mut read2).unwrap();
        assert_eq!(value, val, "Dispatch Code Read: {:?}", code);
    }
}
