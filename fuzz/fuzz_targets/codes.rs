#![no_main]

use arbitrary::Arbitrary;
use dsi_bistream::prelude::*;
use libfuzzer_sys::fuzz_target;

type ReadWord = u32;
type ReadBuffer = u64;

#[derive(Arbitrary, Debug)]
struct FuzzCase {
    commands: Vec<RandomCommand>,
}

#[derive(Arbitrary, Debug)]
enum RandomCommand {
    WriteBits(u64, u8),
    WriteMinimalBinary(u32, u32),
    WriteUnary(u8, bool, bool),
    Gamma(u64, bool, bool),
    Delta(u64, bool, bool),
    Zeta(u32, u8, bool, bool),
}

fuzz_target!(|data: FuzzCase| {
    //println!("{:#4?}", data);
    let mut buffer_be: Vec<u64> = vec![];
    let mut buffer_le: Vec<u64> = vec![];
    let mut writes = vec![];
    // write
    {
        let mut big = BufferedBitStreamWrite::<BE, _>::new(MemWordWriteVec::new(&mut buffer_be));
        let mut little = BufferedBitStreamWrite::<LE, _>::new(MemWordWriteVec::new(&mut buffer_le));

        for command in data.commands.iter() {
            match command {
                RandomCommand::WriteBits(value, n_bits) => {
                    let n_bits = (1 + (*n_bits % 63)) as usize;
                    let value = get_lowest_bits(*value, n_bits);
                    let big_success = big.write_bits(value, n_bits).is_ok();
                    let little_success = little.write_bits(value, n_bits).is_ok();
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::WriteUnary(value, _read_tab, write_tab) => {
                    let (big_success, little_success) = if *write_tab {
                        (
                            big.write_unary::<true>(*value as u64).is_ok(),
                            little.write_unary::<true>(*value as u64).is_ok(),
                        )
                    } else {
                        (
                            big.write_unary::<false>(*value as u64).is_ok(),
                            little.write_unary::<false>(*value as u64).is_ok(),
                        )
                    };
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Gamma(value, _, write_tab) => {
                    let value = (*value).min(u64::MAX - 1);
                    let (big_success, little_success) = if *write_tab {
                        (
                            big.write_gamma::<true>(value).is_ok(),
                            little.write_gamma::<true>(value).is_ok(),
                        )
                    } else {
                        (
                            big.write_gamma::<false>(value).is_ok(),
                            little.write_gamma::<false>(value).is_ok(),
                        )
                    };
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Delta(value, _, write_tab) => {
                    let value = (*value).min(u64::MAX - 1);
                    let (big_success, little_success) = if *write_tab {
                        (
                            big.write_delta::<true, false>(value).is_ok(),
                            little.write_delta::<true, false>(value).is_ok(),
                        )
                    } else {
                        (
                            big.write_delta::<false, false>(value).is_ok(),
                            little.write_delta::<false, false>(value).is_ok(),
                        )
                    };
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::WriteMinimalBinary(value, max) => {
                    let max = (*max).max(1) as u64;
                    let value = (*value as u64) % max;
                    let big_success = big.write_minimal_binary(value, max).is_ok();
                    let little_success = little.write_minimal_binary(value, max).is_ok();
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
                RandomCommand::Zeta(value, k, _, write_tab) => {
                    let value = *value as u64;
                    let k = (*k).max(1).min(7) as u64;

                    let (big_success, little_success) = if *write_tab {
                        (
                            big.write_zeta::<true>(value, k).is_ok(),
                            little.write_zeta::<true>(value, k).is_ok(),
                        )
                    } else {
                        (
                            big.write_zeta::<false>(value, k).is_ok(),
                            little.write_zeta::<false>(value, k).is_ok(),
                        )
                    };
                    assert_eq!(big_success, little_success);
                    writes.push(big_success);
                }
            };
        }
    }
    // read back
    //println!("{:?}", buffer_be);
    //println!("{:?}", buffer_le);
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
        let mut little = UnbufferedBitStreamRead::<LE, _>::new(MemWordRead::new(&buffer_le));
        let mut little_buff =
            BufferedBitStreamRead::<LE, ReadBuffer, _>::new(MemWordRead::new(le_trans));

        for (succ, command) in writes.iter().zip(data.commands.iter()) {
            let pos = big.get_position();
            assert_eq!(pos, little.get_position());
            assert_eq!(pos, big_buff.get_position());
            assert_eq!(pos, little_buff.get_position());

            match command {
                RandomCommand::WriteBits(value, n_bits) => {
                    let n_bits = (1 + (*n_bits % 63)) as usize;
                    let b = big.read_bits(n_bits);
                    let l = little.read_bits(n_bits);
                    let bb = big_buff.read_bits(n_bits);
                    let lb = little_buff.read_bits(n_bits);
                    if *succ {
                        let value = get_lowest_bits(*value, n_bits);
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
                        assert_eq!(pos + n_bits as usize, big.get_position());
                        assert_eq!(pos + n_bits as usize, little.get_position());
                        assert_eq!(pos + n_bits as usize, big_buff.get_position());
                        assert_eq!(pos + n_bits as usize, little_buff.get_position());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_position());
                        assert_eq!(pos, little.get_position());
                        assert_eq!(pos, big_buff.get_position());
                        assert_eq!(pos, little_buff.get_position());
                    }
                }
                RandomCommand::WriteUnary(value, read_tab, _write_tab) => {
                    let (b, l, bb, lb) = if *read_tab {
                        (
                            big.read_unary::<true>(),
                            little.read_unary::<true>(),
                            big_buff.read_unary::<true>(),
                            little_buff.read_unary::<true>(),
                        )
                    } else {
                        (
                            big.read_unary::<false>(),
                            little.read_unary::<false>(),
                            big_buff.read_unary::<false>(),
                            little_buff.read_unary::<false>(),
                        )
                    };
                    if *succ {
                        assert_eq!(b.unwrap(), *value as u64);
                        assert_eq!(l.unwrap(), *value as u64);
                        assert_eq!(bb.unwrap(), *value as u64);
                        assert_eq!(lb.unwrap(), *value as u64);
                        assert_eq!(pos + len_unary::<true>(*value as u64), big.get_position());
                        assert_eq!(
                            pos + len_unary::<true>(*value as u64),
                            little.get_position()
                        );
                        assert_eq!(
                            pos + len_unary::<true>(*value as u64),
                            big_buff.get_position()
                        );
                        assert_eq!(
                            pos + len_unary::<true>(*value as u64),
                            little_buff.get_position()
                        );
                        assert_eq!(pos + len_unary::<false>(*value as u64), big.get_position());
                        assert_eq!(
                            pos + len_unary::<false>(*value as u64),
                            little.get_position()
                        );
                        assert_eq!(
                            pos + len_unary::<false>(*value as u64),
                            big_buff.get_position()
                        );
                        assert_eq!(
                            pos + len_unary::<false>(*value as u64),
                            little_buff.get_position()
                        );
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_position());
                        assert_eq!(pos, little.get_position());
                        assert_eq!(pos, big_buff.get_position());
                        assert_eq!(pos, little_buff.get_position());
                    }
                }
                RandomCommand::Gamma(value, read_tab, _) => {
                    let value = (*value).min(u64::MAX - 1);
                    let (b, l, bb, lb) = if *read_tab {
                        (
                            big.read_gamma::<true>(),
                            little.read_gamma::<true>(),
                            big_buff.read_gamma::<true>(),
                            little_buff.read_gamma::<true>(),
                        )
                    } else {
                        (
                            big.read_gamma::<false>(),
                            little.read_gamma::<false>(),
                            big_buff.read_gamma::<false>(),
                            little_buff.read_gamma::<false>(),
                        )
                    };
                    if *succ {
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(pos + len_gamma::<false>(value), big.get_position());
                        assert_eq!(pos + len_gamma::<false>(value), little.get_position());
                        assert_eq!(pos + len_gamma::<false>(value), big_buff.get_position());
                        assert_eq!(pos + len_gamma::<false>(value), little_buff.get_position());
                        assert_eq!(pos + len_gamma::<true>(value), big.get_position());
                        assert_eq!(pos + len_gamma::<true>(value), little.get_position());
                        assert_eq!(pos + len_gamma::<true>(value), big_buff.get_position());
                        assert_eq!(pos + len_gamma::<true>(value), little_buff.get_position());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_position());
                        assert_eq!(pos, little.get_position());
                        assert_eq!(pos, big_buff.get_position());
                        assert_eq!(pos, little_buff.get_position());
                    }
                }
                RandomCommand::Delta(value, read_tab, _) => {
                    let value = (*value).min(u64::MAX - 1);
                    let (b, l, bb, lb) = if *read_tab {
                        (
                            big.read_delta::<true, false>(),
                            little.read_delta::<true, false>(),
                            big_buff.read_delta::<true, false>(),
                            little_buff.read_delta::<true, false>(),
                        )
                    } else {
                        (
                            big.read_delta::<false, false>(),
                            little.read_delta::<false, false>(),
                            big_buff.read_delta::<false, false>(),
                            little_buff.read_delta::<false, false>(),
                        )
                    };
                    if *succ {
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(pos + len_delta::<true>(value), big.get_position());
                        assert_eq!(pos + len_delta::<true>(value), little.get_position());
                        assert_eq!(pos + len_delta::<true>(value), big_buff.get_position());
                        assert_eq!(pos + len_delta::<true>(value), little_buff.get_position());
                        assert_eq!(pos + len_delta::<false>(value), big.get_position());
                        assert_eq!(pos + len_delta::<false>(value), little.get_position());
                        assert_eq!(pos + len_delta::<false>(value), big_buff.get_position());
                        assert_eq!(pos + len_delta::<false>(value), little_buff.get_position());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_position());
                        assert_eq!(pos, little.get_position());
                        assert_eq!(pos, big_buff.get_position());
                        assert_eq!(pos, little_buff.get_position());
                    }
                }
                RandomCommand::WriteMinimalBinary(value, max) => {
                    let max = (*max).max(1) as u64;
                    let value = (*value as u64) % max;
                    let n_bits = len_minimal_binary(value, max) as u8;
                    let b = big.read_minimal_binary(max);
                    let l = little.read_minimal_binary(max);
                    let bb = big_buff.read_minimal_binary(max);
                    let lb = little_buff.read_minimal_binary(max);
                    if *succ {
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
                        assert_eq!(pos + n_bits as usize, big.get_position());
                        assert_eq!(pos + n_bits as usize, little.get_position());
                        assert_eq!(pos + n_bits as usize, big_buff.get_position());
                        assert_eq!(pos + n_bits as usize, little_buff.get_position());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_position());
                        assert_eq!(pos, little.get_position());
                        assert_eq!(pos, big_buff.get_position());
                        assert_eq!(pos, little_buff.get_position());
                    }
                }
                RandomCommand::Zeta(value, k, read_tab, _) => {
                    let value = *value as u64;
                    let k = (*k).max(1).min(7) as u64;
                    let (b, l, bb, lb) = if *read_tab {
                        (
                            big.read_zeta::<true>(k),
                            little.read_zeta::<true>(k),
                            big_buff.read_zeta::<true>(k),
                            little_buff.read_zeta::<true>(k),
                        )
                    } else {
                        (
                            big.read_zeta::<false>(k),
                            little.read_zeta::<false>(k),
                            big_buff.read_zeta::<false>(k),
                            little_buff.read_zeta::<false>(k),
                        )
                    };
                    if *succ {
                        assert_eq!(bb.unwrap(), value);
                        assert_eq!(lb.unwrap(), value);
                        assert_eq!(b.unwrap(), value);
                        assert_eq!(l.unwrap(), value);
                        assert_eq!(pos + len_zeta::<false>(value, k), big.get_position());
                        assert_eq!(pos + len_zeta::<false>(value, k), little.get_position());
                        assert_eq!(pos + len_zeta::<false>(value, k), big_buff.get_position());
                        assert_eq!(
                            pos + len_zeta::<false>(value, k),
                            little_buff.get_position()
                        );
                        assert_eq!(pos + len_zeta::<true>(value, k), big.get_position());
                        assert_eq!(pos + len_zeta::<true>(value, k), little.get_position());
                        assert_eq!(pos + len_zeta::<true>(value, k), big_buff.get_position());
                        assert_eq!(pos + len_zeta::<true>(value, k), little_buff.get_position());
                    } else {
                        assert!(b.is_err());
                        assert!(l.is_err());
                        assert!(bb.is_err());
                        assert!(lb.is_err());
                        assert_eq!(pos, big.get_position());
                        assert_eq!(pos, little.get_position());
                        assert_eq!(pos, big_buff.get_position());
                        assert_eq!(pos, little_buff.get_position());
                    }
                }
            };
        }
    }
});
