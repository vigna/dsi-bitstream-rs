/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use dsi_bitstream::dispatch::{CodeLen, CodesRead, CodesWrite, FuncCodeReader, FuncCodeWriter};
use dsi_bitstream::prelude::*;

type BW<'a, E> = BufBitWriter<E, MemWordWriterVec<u64, &'a mut Vec<u64>>>;
type BR<'a, E> = BufBitReader<E, MemWordReader<u32, &'a [u32]>>;

#[test]
fn test_codes() {
    test_codes_endianness::<LE>();
    test_codes_endianness::<BE>();
}

fn test_codes_endianness<E: Endianness>()
where
    for<'a> BW<'a, E>: CodesWrite<E>,
    for<'a> BR<'a, E>: CodesRead<E>,
{
    test_binary_min(&[
        (0, 1),
        (0, 2),
        (1, 2),
        (0, 3),
        (1, 3),
        (2, 3),
        (0, u64::MAX),
        (1, u64::MAX),
        (u64::MAX / 2 - 1, u64::MAX),
        (u64::MAX / 2, u64::MAX),
        (u64::MAX / 2 + 1, u64::MAX),
        (u64::MAX - 1, u64::MAX),
    ]);

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

    for k in 1..3 {
        test_codes_and_vals(Codes::ExpGolomb { k }, &[u64::MAX]);
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

    test_golomb(&[
        (0, u64::MAX),
        (u64::MAX / 2, u64::MAX),
        (u64::MAX, u64::MAX),
    ]);
}

fn test_codes_and_vals<E: Endianness>(code: Codes, vals: &[u64])
where
    for<'a> BW<'a, E>: CodesWrite<E>,
    for<'a> BR<'a, E>: CodesRead<E>,
{
    dbg!(code);
    let write_dispatch = FuncCodeWriter::new(code).unwrap();
    let read_dispatch = FuncCodeReader::new(code).unwrap();

    let mut buffer_call = Vec::<u64>::new();
    let mut buffer_disp = Vec::<u64>::new();
    let mut write_call = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer_call));
    let mut write_disp = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer_disp));

    for &val in vals {
        let len = code.write(&mut write_call, val).unwrap();
        assert_eq!(len, code.len(val), "Code Write Len: {:?}", code);
        let len = write_dispatch.write(&mut write_disp, val).unwrap();
        assert_eq!(len, code.len(val), "Dispatch Code Write Len: {:?}", code);
    }

    drop(write_call);
    drop(write_disp);
    let slice_call = unsafe { buffer_call.as_slice().align_to::<u32>().1 };
    let slice_disp = unsafe { buffer_disp.as_slice().align_to::<u32>().1 };

    let mut read_call = BufBitReader::<E, _>::new(MemWordReader::new(slice_call));
    let mut read_disp = BufBitReader::<E, _>::new(MemWordReader::new(slice_disp));

    for &val in vals {
        let value = code.read(&mut read_call).unwrap();
        assert_eq!(value, val, "Code Read: {:?}", code);
        let value = read_dispatch.read(&mut read_disp).unwrap();
        assert_eq!(value, val, "Dispatch Code Read: {:?}", code);
    }
}

fn test_binary_min<'a, 'b, E: Endianness>(n_us: &[(u64, u64)])
where
    for<'c> BW<'c, E>: CodesWrite<E>,
    for<'c> BR<'c, E>: CodesRead<E>,
{
    let mut v = n_us
        .iter()
        .map(|(n, u)| {
            let r = |r: &mut BR<'a, E>| r.read_minimal_binary(*u);
            let w = |w: &mut BW<'b, E>, n: u64| w.write_minimal_binary(n, *u);
            let l = |n| len_minimal_binary(n, *u);
            (*n, r, w, l)
        })
        .collect::<Vec<_>>();
    test_func::<E, BR<'a, E>, BW<'b, E>>(v.as_mut_slice());
}

fn test_golomb<E: Endianness>(x_bs: &[(u64, u64)])
where
    for<'a> BW<'a, E>: CodesWrite<E>,
    for<'a> BR<'a, E>: CodesRead<E>,
{
    let mut buffer1 = Vec::<u64>::new();
    let mut buffer2 = Vec::<u64>::new();
    let mut write_call = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer1));
    let mut write_disp = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer2));

    for &(x, b) in x_bs {
        let len = write_call.write_golomb(x, b).unwrap();
        assert_eq!(len, len_golomb(x, b));
        let len = write_disp.write_golomb(x, b).unwrap();
        assert_eq!(len, len_golomb(x, b));
    }

    drop(write_call);
    drop(write_disp);
    let slice_call = unsafe { buffer1.as_slice().align_to::<u32>().1 };
    let slice_disp = unsafe { buffer2.as_slice().align_to::<u32>().1 };

    let mut read_call = BufBitReader::<E, _>::new(MemWordReader::new(slice_call));
    let mut read_disp = BufBitReader::<E, _>::new(MemWordReader::new(slice_disp));

    for &(x, b) in x_bs {
        let value = read_call.read_golomb(b).unwrap();
        assert_eq!(value, x);
        let value = read_disp.read_golomb(b).unwrap();
        assert_eq!(value, x);
    }
}

fn test_func<'a, 'b, E: Endianness, CR: CodesRead<E>, CW: CodesWrite<E>>(
    x_r_w_l: &mut [(
        u64,
        impl FnMut(&mut BR<'a, E>) -> Result<u64, CR::Error>,
        impl FnMut(&mut BW<'b, E>, u64) -> Result<usize, CW::Error>,
        impl Fn(u64) -> usize,
    )],
) where
    for<'c> BW<'c, E>: CodesWrite<E>,
    for<'c> BR<'c, E>: CodesRead<E>,
{
    let mut buffer1 = Vec::<u64>::new();
    let mut buffer2 = Vec::<u64>::new();
    let mut write_call = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer1));
    let mut write_disp = BufBitWriter::<E, _>::new(MemWordWriterVec::new(&mut buffer2));

    for (x, _, w, l) in x_r_w_l.iter_mut() {
        let len = w(&mut write_call, *x).unwrap();
        assert_eq!(len, l(*x));
        let len = w(&mut write_disp, *x).unwrap();
        assert_eq!(len, l(*x));
    }

    drop(write_call);
    drop(write_disp);
    let slice_call = unsafe { buffer1.as_slice().align_to::<u32>().1 };
    let slice_disp = unsafe { buffer2.as_slice().align_to::<u32>().1 };

    let mut read_call = BufBitReader::<E, _>::new(MemWordReader::new(slice_call));
    let mut read_disp = BufBitReader::<E, _>::new(MemWordReader::new(slice_disp));

    for (x, r, _, l) in x_r_w_l {
        let len = r(&mut read_call).unwrap() as usize;
        assert_eq!(len, l(*x));
        let len = r(&mut read_disp).unwrap() as usize;
        assert_eq!(len, l(*x));
    }
}
