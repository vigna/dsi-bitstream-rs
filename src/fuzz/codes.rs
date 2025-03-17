/*
 * SPDX-FileCopyrightText: 2023 Inria
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

use crate::prelude::*;
use arbitrary::Arbitrary;
use std::io::{Read, Write};

type ReadWord = u32;

const DEBUG: bool = false;

macro_rules! debug {
    ($($arg:tt)*) => {
        if DEBUG {
            print!($($arg)*);
        }
    };
}

macro_rules! debugln {
    ($($arg:tt)*) => {
        if DEBUG {
            println!($($arg)*);
        }
    };
}

#[derive(Arbitrary, Debug, Clone)]
pub struct FuzzCase {
    commands: Vec<RandomCommand>,
}

#[derive(Arbitrary, Debug, Clone)]
enum RandomCommand {
    Bits(u64, usize),
    MinimalBinary(u64, u64),
    Unary(u64),
    Gamma(u64, bool, bool),
    Delta(u64, bool, bool),
    Zeta(u64, usize, bool, bool),
    Golomb(u64, u64),
    Rice(u64, usize),
    ExpGolomb(u64, usize),
    Bytes(Vec<u8>),
    VByte(u64),
    Omega(u64),
    Pi(u64, usize),
}

pub fn harness(data: FuzzCase) {
    let mut data = data;
    for command in &mut data.commands {
        match command {
            RandomCommand::Bits(value, n_bits) => {
                *n_bits = 1 + (*n_bits % 63);
                *value &= (1 << *n_bits) - 1;
            }
            RandomCommand::MinimalBinary(value, max) => {
                *max = (*max).max(1).min(u32::MAX as _);
                *value = (*value) % *max;
            }
            RandomCommand::Unary(value) => {
                *value = (*value).min(300);
            }
            RandomCommand::Gamma(value, _, _) => {
                *value = (*value).min(u64::MAX - 1);
            }
            RandomCommand::Delta(value, _, _) => {
                *value = (*value).min(u64::MAX - 1);
            }
            RandomCommand::Zeta(value, k, _, _) => {
                *value = (*value).min(u32::MAX as u64 - 1);
                *k = (*k).max(1).min(7);
            }
            RandomCommand::Golomb(value, b) => {
                *value = (*value).min(u16::MAX as u64 - 1);
                *b = (*b).max(1).min(20);
            }
            RandomCommand::Rice(value, k) => {
                *value = (*value).min(u16::MAX as u64 - 1);
                *k = (*k).max(0).min(8);
            }
            RandomCommand::ExpGolomb(value, k) => {
                *value = (*value).min(u16::MAX as u64 - 1);
                *k = (*k).max(0).min(8);
            }
            RandomCommand::Bytes(_) => {}
            RandomCommand::VByte(_) => {}
            RandomCommand::Omega(value) => {
                *value = (*value).min(u64::MAX - 1);
            }
            RandomCommand::Pi(value, k) => {
                *value = (*value).min(u32::MAX as u64 - 1);
                *k = (*k).max(1).min(7);
            }
        };
    }

    debugln!("{:#4?}", data);

    let mut buffer_be: Vec<u64> = vec![];
    let mut buffer_le: Vec<u64> = vec![];
    let mut writes = vec![];
    // write
    {
        let mut big = BufBitWriter::<BE, _>::new(MemWordWriterVec::new(&mut buffer_be));
        let mut little = BufBitWriter::<LE, _>::new(MemWordWriterVec::new(&mut buffer_le));

        for command in data.commands.iter() {
            match command {
                RandomCommand::Bits(value, n_bits) => {
                    let big_success = big.write_bits(*value, *n_bits).is_ok();
                    let little_success = little.write_bits(*value, *n_bits).is_ok();
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::MinimalBinary(value, max) => {
                    let big_success = big.write_minimal_binary(*value, *max).is_ok();
                    let little_success = little.write_minimal_binary(*value, *max).is_ok();
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Unary(value) => {
                    let (big_success, little_success) = (
                        big.write_unary(*value as u64).is_ok(),
                        little.write_unary(*value as u64).is_ok(),
                    );
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Gamma(value, _, write_tab) => {
                    let (big_success, little_success) = if *write_tab {
                        (
                            big.write_gamma_param::<true>(*value).is_ok(),
                            little.write_gamma_param::<true>(*value).is_ok(),
                        )
                    } else {
                        (
                            big.write_gamma_param::<false>(*value).is_ok(),
                            little.write_gamma_param::<false>(*value).is_ok(),
                        )
                    };
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Delta(value, _, write_tab) => {
                    let (big_success, little_success) = if *write_tab {
                        (
                            big.write_delta_param::<true, false>(*value).is_ok(),
                            little.write_delta_param::<true, false>(*value).is_ok(),
                        )
                    } else {
                        (
                            big.write_delta_param::<false, false>(*value).is_ok(),
                            little.write_delta_param::<false, false>(*value).is_ok(),
                        )
                    };
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Zeta(value, k, _, write_tab) => {
                    let (big_success, little_success) = if *write_tab {
                        (
                            big.write_zeta_param::<true>(*value, *k).is_ok(),
                            little.write_zeta_param::<true>(*value, *k).is_ok(),
                        )
                    } else {
                        (
                            big.write_zeta_param::<false>(*value, *k).is_ok(),
                            little.write_zeta_param::<false>(*value, *k).is_ok(),
                        )
                    };
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Golomb(value, b) => {
                    let (big_success, little_success) = (
                        big.write_golomb(*value, *b).is_ok(),
                        little.write_golomb(*value, *b).is_ok(),
                    );
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Rice(value, k) => {
                    let (big_success, little_success) = (
                        big.write_rice(*value, *k).is_ok(),
                        little.write_rice(*value, *k).is_ok(),
                    );
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::ExpGolomb(value, k) => {
                    let (big_success, little_success) = (
                        big.write_exp_golomb(*value, *k).is_ok(),
                        little.write_exp_golomb(*value, *k).is_ok(),
                    );
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Bytes(bytes) => {
                    let (big_success, little_success) =
                        (big.write(bytes).is_ok(), little.write(bytes).is_ok());
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::VByte(value) => {
                    let (big_success, little_success) = (
                        big.write_vbyte(*value).is_ok(),
                        little.write_vbyte(*value).is_ok(),
                    );
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Omega(value) => {
                    let (big_success, little_success) = (
                        big.write_omega(*value).is_ok(),
                        little.write_omega(*value).is_ok(),
                    );
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Pi(value, k) => {
                    let (big_success, little_success) = (
                        big.write_pi(*value, *k).is_ok(),
                        little.write_pi(*value, *k).is_ok(),
                    );
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
            };
        }
    }
    // read back
    debug!("BE: ");
    for word in &buffer_be {
        debug!("{:064b} ", *word);
    }
    debug!("\n");
    debug!("LE: ");
    for word in &buffer_le {
        debug!("{:064b} ", *word);
    }
    debug!("\n");
    let be_trans: &[ReadWord] = unsafe {
        core::slice::from_raw_parts(
            buffer_be.as_ptr() as *const ReadWord,
            buffer_be.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
        )
    };
    let le_trans: &[ReadWord] = unsafe {
        core::slice::from_raw_parts(
            buffer_le.as_ptr() as *const ReadWord,
            buffer_le.len() * (core::mem::size_of::<u64>() / core::mem::size_of::<ReadWord>()),
        )
    };
    {
        let mut big = BitReader::<BE, _>::new(MemWordReader::new(&buffer_be));
        let mut big_buff = BufBitReader::<BE, _>::new(MemWordReader::new(be_trans));

        let mut little = BitReader::<LE, _>::new(MemWordReader::new(&buffer_le));
        let mut little_buff = BufBitReader::<LE, _>::new(MemWordReader::new(le_trans));

        for (succ, command) in writes.into_iter().zip(data.commands.into_iter()) {
            let pos = big.bit_pos().unwrap();
            assert_eq!(pos, little.bit_pos().unwrap());
            assert_eq!(pos, big_buff.bit_pos().unwrap());
            assert_eq!(pos, little_buff.bit_pos().unwrap());

            match command {
                RandomCommand::Bits(value, n_bits) => {
                    let b = big.read_bits(n_bits);
                    let l = little.read_bits(n_bits);
                    let bb = big_buff.read_bits(n_bits);
                    let lb = little_buff.read_bits(n_bits);
                    if succ {
                        let b = b.unwrap();
                        let l = l.unwrap();
                        let bb = bb.unwrap();
                        let lb = lb.unwrap();
                        assert_eq!(
                            b,
                            value,
                            "\nread : {:0n$b}\ntruth: {:0n$b}",
                            b,
                            value,
                            n = n_bits as _
                        );
                        assert_eq!(
                            l,
                            value,
                            "\nread : {:0n$b}\ntruth: {:0n$b}",
                            l,
                            value,
                            n = n_bits as _
                        );
                        assert_eq!(
                            bb,
                            value,
                            "\nread : {:0n$b}\ntruth: {:0n$b}",
                            bb,
                            value,
                            n = n_bits as _
                        );
                        assert_eq!(
                            lb,
                            value,
                            "\nread : {:0n$b}\ntruth: {:0n$b}",
                            lb,
                            value,
                            n = n_bits as _
                        );
                        assert_eq!(pos + n_bits as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + n_bits as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + n_bits as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(pos + n_bits as u64, little_buff.bit_pos().unwrap());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }

                RandomCommand::MinimalBinary(value, max) => {
                    let n_bits = len_minimal_binary(value, max) as u8;
                    let b = big.read_minimal_binary(max);
                    let l = little.read_minimal_binary(max);
                    let bb = big_buff.read_minimal_binary(max);
                    let lb = little_buff.read_minimal_binary(max);
                    if succ {
                        let b = b.unwrap();
                        let l = l.unwrap();
                        let bb = bb.unwrap();
                        let lb = lb.unwrap();
                        assert_eq!(
                            b,
                            value,
                            "\nread : {:0n$b}\ntruth: {:0n$b}",
                            b,
                            value,
                            n = n_bits as _
                        );
                        assert_eq!(
                            l,
                            value,
                            "\nread : {:0n$b}\ntruth: {:0n$b}",
                            l,
                            value,
                            n = n_bits as _
                        );
                        assert_eq!(
                            bb,
                            value,
                            "\nread : {:0n$b}\ntruth: {:0n$b}",
                            bb,
                            value,
                            n = n_bits as _
                        );
                        assert_eq!(
                            lb,
                            value,
                            "\nread : {:0n$b}\ntruth: {:0n$b}",
                            lb,
                            value,
                            n = n_bits as _
                        );
                        assert_eq!(pos + n_bits as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + n_bits as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + n_bits as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(pos + n_bits as u64, little_buff.bit_pos().unwrap());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }

                RandomCommand::Unary(value) => {
                    let (b, l, bb, lb) = (
                        big.read_unary(),
                        little.read_unary(),
                        big_buff.read_unary(),
                        little_buff.read_unary(),
                    );
                    if succ {
                        assert_eq!(b.unwrap(), value as u64);
                        assert_eq!(l.unwrap(), value as u64);
                        assert_eq!(bb.unwrap(), value as u64);
                        assert_eq!(lb.unwrap(), value as u64);
                        assert_eq!(pos + value as u64 + 1, big.bit_pos().unwrap());
                        assert_eq!(pos + value as u64 + 1, little.bit_pos().unwrap());
                        assert_eq!(pos + value as u64 + 1, big_buff.bit_pos().unwrap());
                        assert_eq!(pos + value as u64 + 1, little_buff.bit_pos().unwrap());

                        assert_eq!(pos + value as u64 + 1, big.bit_pos().unwrap());
                        assert_eq!(pos + value as u64 + 1, little.bit_pos().unwrap());
                        assert_eq!(pos + value as u64 + 1, big_buff.bit_pos().unwrap());
                        assert_eq!(pos + value as u64 + 1, little_buff.bit_pos().unwrap());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }

                RandomCommand::Gamma(value, read_tab, _) => {
                    let (b, l, bb, lb) = if read_tab {
                        (
                            big.read_gamma_param::<true>(),
                            little.read_gamma_param::<true>(),
                            big_buff.read_gamma_param::<true>(),
                            little_buff.read_gamma_param::<true>(),
                        )
                    } else {
                        (
                            big.read_gamma_param::<false>(),
                            little.read_gamma_param::<false>(),
                            big_buff.read_gamma_param::<false>(),
                            little_buff.read_gamma_param::<false>(),
                        )
                    };
                    if succ {
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(
                            pos + len_gamma_param::<false>(value) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_gamma_param::<false>(value) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_gamma_param::<false>(value) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_gamma_param::<false>(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_gamma_param::<true>(value) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_gamma_param::<true>(value) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_gamma_param::<true>(value) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_gamma_param::<true>(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }

                RandomCommand::Delta(value, read_tab, _) => {
                    let (b, l, bb, lb) = if read_tab {
                        (
                            big.read_delta_param::<true, false>(),
                            little.read_delta_param::<true, false>(),
                            big_buff.read_delta_param::<true, false>(),
                            little_buff.read_delta_param::<true, false>(),
                        )
                    } else {
                        (
                            big.read_delta_param::<false, false>(),
                            little.read_delta_param::<false, false>(),
                            big_buff.read_delta_param::<false, false>(),
                            little_buff.read_delta_param::<false, false>(),
                        )
                    };
                    if succ {
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(
                            pos + len_delta_param::<true, true>(value) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, true>(value) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, true>(value) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, true>(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, true>(value) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, true>(value) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, true>(value) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, true>(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, false>(value) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, false>(value) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, false>(value) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, false>(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, false>(value) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, false>(value) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, false>(value) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, false>(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }

                RandomCommand::Zeta(value, k, read_tab, _) => {
                    let (b, l, bb, lb) = if k == 3 {
                        if read_tab {
                            (
                                big.read_zeta3_param::<true>(),
                                little.read_zeta3_param::<true>(),
                                big_buff.read_zeta3_param::<true>(),
                                little_buff.read_zeta3_param::<true>(),
                            )
                        } else {
                            (
                                big.read_zeta3_param::<false>(),
                                little.read_zeta3_param::<false>(),
                                big_buff.read_zeta3_param::<false>(),
                                little_buff.read_zeta3_param::<false>(),
                            )
                        }
                    } else {
                        (
                            big.read_zeta_param(k),
                            little.read_zeta_param(k),
                            big_buff.read_zeta_param(k),
                            little_buff.read_zeta_param(k),
                        )
                    };
                    if succ {
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(
                            pos + len_zeta_param::<false>(value, k) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_zeta_param::<false>(value, k) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_zeta_param::<false>(value, k) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_zeta_param::<false>(value, k) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_zeta_param::<true>(value, k) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_zeta_param::<true>(value, k) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_zeta_param::<true>(value, k) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_zeta_param::<true>(value, k) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }
                RandomCommand::Golomb(value, b_par) => {
                    let (b, l, bb, lb) = (
                        big.read_golomb(b_par),
                        little.read_golomb(b_par),
                        big_buff.read_golomb(b_par),
                        little_buff.read_golomb(b_par),
                    );
                    if succ {
                        assert_eq!(b.unwrap(), value as u64);
                        assert_eq!(l.unwrap(), value as u64);
                        assert_eq!(bb.unwrap(), value as u64);
                        assert_eq!(lb.unwrap(), value as u64);
                        assert_eq!(
                            pos + len_golomb(value, b_par) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_golomb(value, b_par) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_golomb(value, b_par) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_golomb(value, b_par) as u64,
                            little_buff.bit_pos().unwrap()
                        );

                        assert_eq!(
                            pos + len_golomb(value, b_par) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_golomb(value, b_par) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_golomb(value, b_par) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_golomb(value, b_par) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }
                RandomCommand::Rice(value, k) => {
                    let (b, l, bb, lb) = (
                        big.read_rice(k),
                        little.read_rice(k),
                        big_buff.read_rice(k),
                        little_buff.read_rice(k),
                    );
                    if succ {
                        assert_eq!(b.unwrap(), value as u64);
                        assert_eq!(l.unwrap(), value as u64);
                        assert_eq!(bb.unwrap(), value as u64);
                        assert_eq!(lb.unwrap(), value as u64);
                        assert_eq!(pos + len_rice(value, k) as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + len_rice(value, k) as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + len_rice(value, k) as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(
                            pos + len_rice(value, k) as u64,
                            little_buff.bit_pos().unwrap()
                        );

                        assert_eq!(pos + len_rice(value, k) as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + len_rice(value, k) as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + len_rice(value, k) as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(
                            pos + len_rice(value, k) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }
                RandomCommand::ExpGolomb(value, k) => {
                    let (b, l, bb, lb) = (
                        big.read_exp_golomb(k),
                        little.read_exp_golomb(k),
                        big_buff.read_exp_golomb(k),
                        little_buff.read_exp_golomb(k),
                    );
                    if succ {
                        assert_eq!(b.unwrap(), value as u64);
                        assert_eq!(l.unwrap(), value as u64);
                        assert_eq!(bb.unwrap(), value as u64);
                        assert_eq!(lb.unwrap(), value as u64);
                        assert_eq!(
                            pos + len_exp_golomb(value, k) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_exp_golomb(value, k) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_exp_golomb(value, k) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_exp_golomb(value, k) as u64,
                            little_buff.bit_pos().unwrap()
                        );

                        assert_eq!(
                            pos + len_exp_golomb(value, k) as u64,
                            big.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_exp_golomb(value, k) as u64,
                            little.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_exp_golomb(value, k) as u64,
                            big_buff.bit_pos().unwrap()
                        );
                        assert_eq!(
                            pos + len_exp_golomb(value, k) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }
                RandomCommand::VByte(value) => {
                    let (b, l, bb, lb) = (
                        big.read_vbyte(),
                        little.read_vbyte(),
                        big_buff.read_vbyte(),
                        little_buff.read_vbyte(),
                    );
                    if succ {
                        assert_eq!(b.unwrap(), value as u64, "b");
                        assert_eq!(l.unwrap(), value as u64, "l");
                        assert_eq!(bb.unwrap(), value as u64, "bb");
                        assert_eq!(lb.unwrap(), value as u64, "lb");
                        assert_eq!(pos + len_vbyte(value) as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + len_vbyte(value) as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + len_vbyte(value) as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(
                            pos + len_vbyte(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );

                        assert_eq!(pos + len_vbyte(value) as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + len_vbyte(value) as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + len_vbyte(value) as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(
                            pos + len_vbyte(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }
                RandomCommand::Bytes(bytes) => {
                    let mut b_buffer = vec![0; bytes.len()];
                    let b = big.read_exact(&mut b_buffer);
                    let mut l_buffer = vec![0; bytes.len()];
                    let l = little.read_exact(&mut l_buffer);
                    let mut bb_buffer = vec![0; bytes.len()];
                    let bb = big_buff.read_exact(&mut bb_buffer);
                    let mut lb_buffer = vec![0; bytes.len()];
                    let lb = little_buff.read_exact(&mut lb_buffer);

                    if succ {
                        assert_eq!(&b_buffer, &bytes);
                        assert_eq!(&l_buffer, &bytes);
                        assert_eq!(&bb_buffer, &bytes);
                        assert_eq!(&lb_buffer, &bytes);
                        assert_eq!(pos + bytes.len() as u64 * 8, big.bit_pos().unwrap());
                        assert_eq!(pos + bytes.len() as u64 * 8, little.bit_pos().unwrap());
                        assert_eq!(pos + bytes.len() as u64 * 8, big_buff.bit_pos().unwrap());
                        assert_eq!(pos + bytes.len() as u64 * 8, little_buff.bit_pos().unwrap());

                        assert_eq!(pos + bytes.len() as u64 * 8, big.bit_pos().unwrap());
                        assert_eq!(pos + bytes.len() as u64 * 8, little.bit_pos().unwrap());
                        assert_eq!(pos + bytes.len() as u64 * 8, big_buff.bit_pos().unwrap());
                        assert_eq!(pos + bytes.len() as u64 * 8, little_buff.bit_pos().unwrap());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }
                RandomCommand::Omega(value) => {
                    let (b, l, bb, lb) = (
                        big.read_omega(),
                        little.read_omega(),
                        big_buff.read_omega(),
                        little_buff.read_omega(),
                    );
                    if succ {
                        assert_eq!(b.unwrap(), value as u64, "b");
                        assert_eq!(l.unwrap(), value as u64, "l");
                        assert_eq!(bb.unwrap(), value as u64, "bb");
                        assert_eq!(lb.unwrap(), value as u64, "lb");
                        assert_eq!(pos + len_omega(value) as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + len_omega(value) as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + len_omega(value) as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(
                            pos + len_omega(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );

                        assert_eq!(pos + len_omega(value) as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + len_omega(value) as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + len_omega(value) as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(
                            pos + len_omega(value) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }
                RandomCommand::Pi(value, k) => {
                    let (b, l, bb, lb) = (
                        big.read_pi(k),
                        little.read_pi(k),
                        big_buff.read_pi(k),
                        little_buff.read_pi(k),
                    );
                    if succ {
                        assert_eq!(b.unwrap(), value as u64);
                        assert_eq!(l.unwrap(), value as u64);
                        assert_eq!(bb.unwrap(), value as u64);
                        assert_eq!(lb.unwrap(), value as u64);
                        assert_eq!(pos + len_pi(value, k) as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + len_pi(value, k) as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + len_pi(value, k) as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(
                            pos + len_pi(value, k) as u64,
                            little_buff.bit_pos().unwrap()
                        );

                        assert_eq!(pos + len_pi(value, k) as u64, big.bit_pos().unwrap());
                        assert_eq!(pos + len_pi(value, k) as u64, little.bit_pos().unwrap());
                        assert_eq!(pos + len_pi(value, k) as u64, big_buff.bit_pos().unwrap());
                        assert_eq!(
                            pos + len_pi(value, k) as u64,
                            little_buff.bit_pos().unwrap()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.bit_pos().unwrap());
                        assert_eq!(pos, little.bit_pos().unwrap());
                        assert_eq!(pos, big_buff.bit_pos().unwrap());
                        assert_eq!(pos, little_buff.bit_pos().unwrap());
                    }
                }
            };
        }
    }
}
