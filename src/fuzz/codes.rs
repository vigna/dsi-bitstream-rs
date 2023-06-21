use crate::prelude::*;
use arbitrary::Arbitrary;

type ReadWord = u32;
type ReadBuffer = u64;

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
pub enum RandomCommand {
    Bits(u64, usize),
    MinimalBinary(u64, u64),
    Unary(u64, bool, bool),
    Gamma(u64, bool, bool),
    Delta(u64, bool, bool),
    Zeta(u64, u64, bool, bool),
}

pub fn codes(data: FuzzCase) {
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
            RandomCommand::Unary(value, _, _) => {
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
        };
    }

    debugln!("{:#4?}", data);

    let mut buffer_be: Vec<u64> = vec![];
    let mut buffer_le: Vec<u64> = vec![];
    let mut writes = vec![];
    // write
    {
        let mut big = BufferedBitStreamWrite::<BE, _>::new(MemWordWriteVec::new(&mut buffer_be));
        let mut little = BufferedBitStreamWrite::<LE, _>::new(MemWordWriteVec::new(&mut buffer_le));

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
                RandomCommand::Unary(value, _read_tab, write_tab) => {
                    let (big_success, little_success) = if *write_tab {
                        (
                            big.write_unary_param::<true>(*value as u64).is_ok(),
                            little.write_unary_param::<true>(*value as u64).is_ok(),
                        )
                    } else {
                        (
                            big.write_unary_param::<false>(*value as u64).is_ok(),
                            little.write_unary_param::<false>(*value as u64).is_ok(),
                        )
                    };
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
        let mut big = UnbufferedBitStreamRead::<BE, _>::new(MemWordRead::new(&buffer_be));
        let mut big_buff =
            BufferedBitStreamRead::<BE, ReadBuffer, _>::new(MemWordRead::new(be_trans));
        let mut big_buff_skip =
            BufferedBitStreamRead::<BE, ReadBuffer, _>::new(MemWordRead::new(be_trans));

        let mut little = UnbufferedBitStreamRead::<LE, _>::new(MemWordRead::new(&buffer_le));
        let mut little_buff =
            BufferedBitStreamRead::<LE, ReadBuffer, _>::new(MemWordRead::new(le_trans));
        let mut little_buff_skip =
            BufferedBitStreamRead::<LE, ReadBuffer, _>::new(MemWordRead::new(le_trans));

        for (succ, command) in writes.into_iter().zip(data.commands.into_iter()) {
            let pos = big.get_pos();
            assert_eq!(pos, little.get_pos());
            assert_eq!(pos, big_buff.get_pos());
            assert_eq!(pos, little_buff.get_pos());

            match command {
                RandomCommand::Bits(value, n_bits) => {
                    let b = big.read_bits(n_bits);
                    let l = little.read_bits(n_bits);
                    let bb = big_buff.read_bits(n_bits);
                    let lb = little_buff.read_bits(n_bits);
                    if succ {
                        big_buff_skip.skip_bits(n_bits).unwrap();
                        little_buff_skip.skip_bits(n_bits).unwrap();
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
                        assert_eq!(pos + n_bits as usize, big.get_pos());
                        assert_eq!(pos + n_bits as usize, little.get_pos());
                        assert_eq!(pos + n_bits as usize, big_buff.get_pos());
                        assert_eq!(pos + n_bits as usize, little_buff.get_pos());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_pos());
                        assert_eq!(pos, little.get_pos());
                        assert_eq!(pos, big_buff.get_pos());
                        assert_eq!(pos, little_buff.get_pos());
                    }
                }

                RandomCommand::MinimalBinary(value, max) => {
                    let n_bits = len_minimal_binary(value, max) as u8;
                    let b = big.read_minimal_binary(max);
                    let l = little.read_minimal_binary(max);
                    let bb = big_buff.read_minimal_binary(max);
                    let lb = little_buff.read_minimal_binary(max);
                    if succ {
                        assert_eq!(
                            big_buff_skip
                                .skip_minimal_binary(max, 1)
                                .unwrap_or(usize::MAX),
                            n_bits as _
                        );
                        assert_eq!(
                            little_buff_skip
                                .skip_minimal_binary(max, 1)
                                .unwrap_or(usize::MAX),
                            n_bits as _
                        );
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
                        assert_eq!(pos + n_bits as usize, big.get_pos());
                        assert_eq!(pos + n_bits as usize, little.get_pos());
                        assert_eq!(pos + n_bits as usize, big_buff.get_pos());
                        assert_eq!(pos + n_bits as usize, little_buff.get_pos());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_pos());
                        assert_eq!(pos, little.get_pos());
                        assert_eq!(pos, big_buff.get_pos());
                        assert_eq!(pos, little_buff.get_pos());
                    }
                }

                RandomCommand::Unary(value, read_tab, _write_tab) => {
                    let (b, l, bb, lb) = if read_tab {
                        (
                            big.read_unary_param::<true>(),
                            little.read_unary_param::<true>(),
                            big_buff.read_unary_param::<true>(),
                            little_buff.read_unary_param::<true>(),
                        )
                    } else {
                        (
                            big.read_unary_param::<false>(),
                            little.read_unary_param::<false>(),
                            big_buff.read_unary_param::<false>(),
                            little_buff.read_unary_param::<false>(),
                        )
                    };
                    if succ {
                        assert_eq!(
                            big_buff_skip.skip_unary(1).unwrap_or(usize::MAX),
                            len_unary_param::<false>(value)
                        );
                        assert_eq!(
                            little_buff_skip.skip_unary(1).unwrap_or(usize::MAX),
                            len_unary_param::<false>(value)
                        );
                        assert_eq!(b.unwrap(), value as u64);
                        assert_eq!(l.unwrap(), value as u64);
                        assert_eq!(bb.unwrap(), value as u64);
                        assert_eq!(lb.unwrap(), value as u64);
                        assert_eq!(pos + len_unary_param::<true>(value as u64), big.get_pos());
                        assert_eq!(
                            pos + len_unary_param::<true>(value as u64),
                            little.get_pos()
                        );
                        assert_eq!(
                            pos + len_unary_param::<true>(value as u64),
                            big_buff.get_pos()
                        );
                        assert_eq!(
                            pos + len_unary_param::<true>(value as u64),
                            little_buff.get_pos()
                        );
                        assert_eq!(pos + len_unary_param::<false>(value as u64), big.get_pos());
                        assert_eq!(
                            pos + len_unary_param::<false>(value as u64),
                            little.get_pos()
                        );
                        assert_eq!(
                            pos + len_unary_param::<false>(value as u64),
                            big_buff.get_pos()
                        );
                        assert_eq!(
                            pos + len_unary_param::<false>(value as u64),
                            little_buff.get_pos()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_pos());
                        assert_eq!(pos, little.get_pos());
                        assert_eq!(pos, big_buff.get_pos());
                        assert_eq!(pos, little_buff.get_pos());
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
                        assert_eq!(
                            big_buff_skip.skip_gamma(1).unwrap_or(usize::MAX),
                            len_gamma_param::<false>(value)
                        );
                        assert_eq!(
                            little_buff_skip.skip_gamma(1).unwrap_or(usize::MAX),
                            len_gamma_param::<false>(value)
                        );
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(pos + len_gamma_param::<false>(value), big.get_pos());
                        assert_eq!(pos + len_gamma_param::<false>(value), little.get_pos());
                        assert_eq!(pos + len_gamma_param::<false>(value), big_buff.get_pos());
                        assert_eq!(pos + len_gamma_param::<false>(value), little_buff.get_pos());
                        assert_eq!(pos + len_gamma_param::<true>(value), big.get_pos());
                        assert_eq!(pos + len_gamma_param::<true>(value), little.get_pos());
                        assert_eq!(pos + len_gamma_param::<true>(value), big_buff.get_pos());
                        assert_eq!(pos + len_gamma_param::<true>(value), little_buff.get_pos());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_pos());
                        assert_eq!(pos, little.get_pos());
                        assert_eq!(pos, big_buff.get_pos());
                        assert_eq!(pos, little_buff.get_pos());
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
                        assert_eq!(
                            big_buff_skip.skip_delta(1).unwrap_or(usize::MAX),
                            len_delta_param::<false, false>(value)
                        );
                        assert_eq!(
                            little_buff_skip.skip_delta(1).unwrap_or(usize::MAX),
                            len_delta_param::<false, false>(value)
                        );
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(pos + len_delta_param::<true, true>(value), big.get_pos());
                        assert_eq!(pos + len_delta_param::<true, true>(value), little.get_pos());
                        assert_eq!(
                            pos + len_delta_param::<true, true>(value),
                            big_buff.get_pos()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, true>(value),
                            little_buff.get_pos()
                        );
                        assert_eq!(pos + len_delta_param::<false, true>(value), big.get_pos());
                        assert_eq!(
                            pos + len_delta_param::<false, true>(value),
                            little.get_pos()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, true>(value),
                            big_buff.get_pos()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, true>(value),
                            little_buff.get_pos()
                        );
                        assert_eq!(pos + len_delta_param::<true, false>(value), big.get_pos());
                        assert_eq!(
                            pos + len_delta_param::<true, false>(value),
                            little.get_pos()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, false>(value),
                            big_buff.get_pos()
                        );
                        assert_eq!(
                            pos + len_delta_param::<true, false>(value),
                            little_buff.get_pos()
                        );
                        assert_eq!(pos + len_delta_param::<false, false>(value), big.get_pos());
                        assert_eq!(
                            pos + len_delta_param::<false, false>(value),
                            little.get_pos()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, false>(value),
                            big_buff.get_pos()
                        );
                        assert_eq!(
                            pos + len_delta_param::<false, false>(value),
                            little_buff.get_pos()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_pos());
                        assert_eq!(pos, little.get_pos());
                        assert_eq!(pos, big_buff.get_pos());
                        assert_eq!(pos, little_buff.get_pos());
                    }
                }

                RandomCommand::Zeta(value, k, read_tab, _) => {
                    let (b, l, bb, lb) = if read_tab {
                        (
                            big.read_zeta_param::<true>(k),
                            little.read_zeta_param::<true>(k),
                            big_buff.read_zeta_param::<true>(k),
                            little_buff.read_zeta_param::<true>(k),
                        )
                    } else {
                        (
                            big.read_zeta_param::<false>(k),
                            little.read_zeta_param::<false>(k),
                            big_buff.read_zeta_param::<false>(k),
                            little_buff.read_zeta_param::<false>(k),
                        )
                    };
                    if succ {
                        assert_eq!(
                            big_buff_skip.skip_zeta(k, 1).unwrap_or(usize::MAX),
                            len_zeta_param::<false>(value, k)
                        );
                        assert_eq!(
                            little_buff_skip.skip_zeta(k, 1).unwrap_or(usize::MAX),
                            len_zeta_param::<false>(value, k)
                        );
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(pos + len_zeta_param::<false>(value, k), big.get_pos());
                        assert_eq!(pos + len_zeta_param::<false>(value, k), little.get_pos());
                        assert_eq!(pos + len_zeta_param::<false>(value, k), big_buff.get_pos());
                        assert_eq!(
                            pos + len_zeta_param::<false>(value, k),
                            little_buff.get_pos()
                        );
                        assert_eq!(pos + len_zeta_param::<true>(value, k), big.get_pos());
                        assert_eq!(pos + len_zeta_param::<true>(value, k), little.get_pos());
                        assert_eq!(pos + len_zeta_param::<true>(value, k), big_buff.get_pos());
                        assert_eq!(
                            pos + len_zeta_param::<true>(value, k),
                            little_buff.get_pos()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_pos());
                        assert_eq!(pos, little.get_pos());
                        assert_eq!(pos, big_buff.get_pos());
                        assert_eq!(pos, little_buff.get_pos());
                    }
                }
            };
        }
    }
}
