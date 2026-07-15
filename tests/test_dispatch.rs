/*
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! Behavioral coverage for the hand-maintained code-dispatch matrices: the
//! dynamic `Codes` dispatch (dispatch::codes), the function-pointer tables
//! (dispatch::dynamic: FuncCodeReader/Writer/Len) and the const dispatch
//! (dispatch::static: ConstCode).
//!
//! The dynamic and function-pointer matrices are cross-validated against each
//! other: for each code the two are encoded into *separate* buffers, the
//! codewords and lengths must be byte-for-byte identical, and each buffer is
//! read back through the *other* matrix. A wrong arm in either matrix that
//! diverges from the other is therefore caught (a byte mismatch or a wrong
//! decoded value), not hidden by reading each buffer with its own matching arm.
#![cfg(feature = "alloc")]

use dsi_bitstream::dispatch::{
    CodeLen, Codes, ConstCode, DynamicCodeRead, DynamicCodeWrite, FuncCodeLen, FuncCodeReader,
    FuncCodeWriter, StaticCodeRead, StaticCodeWrite, code_consts,
};
use dsi_bitstream::prelude::*;

// A representative code from every family, including several parameter values
// for the parameterized codes and both VByte byte orders.
const CODES: &[Codes] = &[
    Codes::Unary,
    Codes::Gamma,
    Codes::Delta,
    Codes::Omega,
    Codes::VByteLe,
    Codes::VByteBe,
    Codes::Zeta(1),
    Codes::Zeta(2),
    Codes::Zeta(3),
    Codes::Zeta(6),
    Codes::Pi(0),
    Codes::Pi(1),
    Codes::Pi(3),
    Codes::Golomb(1),
    Codes::Golomb(3),
    Codes::Golomb(7),
    Codes::ExpGolomb(0),
    Codes::ExpGolomb(2),
    Codes::ExpGolomb(5),
    Codes::Rice(0),
    Codes::Rice(3),
    Codes::Rice(7),
];
const VALUES: &[u64] = &[0, 1, 2, 7, 100, 1000, 100_000];

fn encode<E, F>(f: F) -> (usize, Vec<u64>)
where
    E: Endianness,
    BufBitWriter<E, MemWordWriterVec<u64, Vec<u64>>>: BitWrite<E> + CodesWrite<E>,
    F: FnOnce(&mut BufBitWriter<E, MemWordWriterVec<u64, Vec<u64>>>) -> usize,
{
    let mut w = BufBitWriter::<E, _>::new(MemWordWriterVec::new(Vec::<u64>::new()));
    let bits = f(&mut w);
    (bits, w.into_inner().unwrap().into_inner())
}

fn check_matrix<E: Endianness>()
where
    BufBitWriter<E, MemWordWriterVec<u64, Vec<u64>>>: BitWrite<E> + CodesWrite<E>,
    BufBitReader<E, MemWordReader<u64, Vec<u64>, true>>: BitRead<E> + CodesRead<E>,
{
    for &code in CODES {
        let fr = FuncCodeReader::<E, _>::new(code).unwrap();
        let fw = FuncCodeWriter::<E, _>::new(code).unwrap();
        let fl = FuncCodeLen::new(code).unwrap();
        for &n in VALUES {
            let (bits_dyn, buf_dyn) =
                encode::<E, _>(|w| DynamicCodeWrite::write(&code, w, n).unwrap());
            let (bits_fw, buf_fw) = encode::<E, _>(|w| StaticCodeWrite::write(&fw, w, n).unwrap());

            // The two writer matrices must emit identical codewords and lengths.
            assert_eq!(
                bits_dyn, bits_fw,
                "writer length mismatch for {code:?} n={n}"
            );
            assert_eq!(
                buf_dyn, buf_fw,
                "writer codeword mismatch for {code:?} n={n}"
            );
            assert_eq!(
                bits_dyn,
                code.len(n),
                "Codes::len mismatch for {code:?} n={n}"
            );
            assert_eq!(
                bits_dyn,
                fl.get_func()(n),
                "FuncCodeLen mismatch for {code:?} n={n}"
            );

            // Cross-read: read each buffer with the *other* matrix's reader.
            let mut r_dyn = BufBitReader::<E, _>::new(MemWordReader::new_inf(buf_dyn));
            assert_eq!(
                StaticCodeRead::read(&fr, &mut r_dyn).unwrap(),
                n,
                "FuncCodeReader on dynamic-encoded buffer for {code:?} n={n}"
            );
            let mut r_fw = BufBitReader::<E, _>::new(MemWordReader::new_inf(buf_fw));
            assert_eq!(
                DynamicCodeRead::read(&code, &mut r_fw).unwrap(),
                n,
                "Codes::read on func-encoded buffer for {code:?} n={n}"
            );
        }
    }
}

#[test]
fn dispatch_matrix_be() {
    check_matrix::<BE>();
}

#[test]
fn dispatch_matrix_le() {
    check_matrix::<LE>();
}

#[test]
fn const_code_roundtrips() {
    const N: u64 = 100;
    // One representative const per family; ConstCode delegates through the
    // static-dispatch matrices, so a wrong const->code mapping is caught here.
    macro_rules! rt {
        ($c:expr) => {{
            let code = ConstCode::<{ $c }>;
            let mut w = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(Vec::<u64>::new()));
            let bits = StaticCodeWrite::write(&code, &mut w, N).unwrap();
            let buf = w.into_inner().unwrap().into_inner();
            assert_eq!(bits, code.len(N), "ConstCode len mismatch for const {}", $c);
            let mut r = BufBitReader::<BE, _>::new(MemWordReader::new_inf(buf));
            assert_eq!(
                StaticCodeRead::read(&code, &mut r).unwrap(),
                N,
                "ConstCode roundtrip mismatch for const {}",
                $c
            );
        }};
    }
    rt!(code_consts::UNARY);
    rt!(code_consts::GAMMA);
    rt!(code_consts::DELTA);
    rt!(code_consts::OMEGA);
    rt!(code_consts::VBYTE_BE);
    rt!(code_consts::VBYTE_LE);
    rt!(code_consts::ZETA3);
}
