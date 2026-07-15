/*
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

//! std::io::{Read,Write} adapters must report the bytes actually transferred,
//! must not lose readable bytes on a narrow-word backend near EOF, and must
//! leave the writer in a clean state on a backend error so a write_all retry is
//! safe and produces exactly the same output as an uninterrupted write.
#![cfg(feature = "std")]

use dsi_bitstream::prelude::*;
use dsi_bitstream::traits::{BE, Endianness, LE, WordError, WordWrite};

/// A `WordWrite` (u8 words) that fails on the `fail_at`-th `write_word` call and
/// records every word it accepts.
struct FailOnce {
    fail_at: usize,
    calls: usize,
    words: Vec<u8>,
}
impl FailOnce {
    fn new(fail_at: usize) -> Self {
        Self {
            fail_at,
            calls: 0,
            words: Vec::new(),
        }
    }
}
impl WordWrite for FailOnce {
    type Error = WordError;
    type Word = u8;
    fn write_word(&mut self, word: u8) -> Result<(), Self::Error> {
        let c = self.calls;
        self.calls += 1;
        if c == self.fail_at {
            return Err(WordError::UnexpectedEof { word_pos: c });
        }
        self.words.push(word);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn read_returns_partial_without_losing_bytes_be() {
    // Strict u8-word backend with only 3 bytes; a chunked read_bits(64) would
    // consume all three words, error, and copy zero bytes (data loss). The
    // byte-by-byte adapter returns all 3.
    let data: [u8; 3] = [0xAA, 0xBB, 0xCC];
    let mut r = BufBitReader::<BE, _>::new(MemWordReader::new(&data));
    let mut buf = [0u8; 8];
    let n = std::io::Read::read(&mut r, &mut buf).unwrap();
    assert_eq!(n, 3);
    assert_eq!(&buf[..3], &[0xAA, 0xBB, 0xCC]);
}

#[test]
fn read_returns_partial_without_losing_bytes_le() {
    let data: [u8; 3] = [0xAA, 0xBB, 0xCC];
    let mut r = BufBitReader::<LE, _>::new(MemWordReader::new(&data));
    let mut buf = [0u8; 8];
    let n = std::io::Read::read(&mut r, &mut buf).unwrap();
    assert_eq!(n, 3);
    assert_eq!(&buf[..3], &[0xAA, 0xBB, 0xCC]);
}

#[test]
fn write_errors_when_no_bytes_accepted_and_stays_clean() {
    // u8 words: the first byte fills a word and flushes, which fails; no byte
    // was accepted, so write returns Err. Because the adapter restores the
    // buffer state, flushing the writer afterwards does not panic.
    let mut w = BufBitWriter::<BE, _>::new(FailOnce::new(0));
    assert!(std::io::Write::write(&mut w, &[1, 2, 3]).is_err());
    let backend = w.into_inner().unwrap();
    assert!(backend.words.is_empty());
}

fn recover<E: Endianness>()
where
    BufBitWriter<E, FailOnce>: BitWrite<E> + std::io::Write,
{
    let data = [0x11u8, 0x22, 0x33, 0x44, 0x55];

    // Reference: the same 4-bit prefix + bytes written to a backend that never
    // fails, giving the correct sequence of accepted words.
    let mut w_ref = BufBitWriter::<E, _>::new(FailOnce::new(usize::MAX));
    w_ref.write_bits(0b1010, 4).unwrap();
    std::io::Write::write_all(&mut w_ref, &data).unwrap();
    let expected = w_ref.into_inner().unwrap().words;

    // Fail-once: a write_all-style retry loop must reproduce exactly the same
    // words (no loss, no duplication) because the adapter restores the writer on
    // the transient error.
    let mut w = BufBitWriter::<E, _>::new(FailOnce::new(1));
    w.write_bits(0b1010, 4).unwrap();
    let mut pos = 0;
    let mut attempts = 0;
    while pos < data.len() {
        attempts += 1;
        assert!(attempts < 1000, "no forward progress");
        match std::io::Write::write(&mut w, &data[pos..]) {
            Ok(0) => panic!("write returned 0"),
            Ok(n) => pos += n,
            Err(_) => {} // transient; writer restored, retry from pos
        }
    }
    let got = w.into_inner().unwrap().words;
    assert_eq!(
        got, expected,
        "fail-once recovery must match an uninterrupted write"
    );
}

#[test]
fn write_recovers_after_transient_backend_error_be() {
    recover::<BE>();
}

#[test]
fn write_recovers_after_transient_backend_error_le() {
    recover::<LE>();
}

#[test]
fn read_after_exhaustion_errors_instead_of_silent_eof() {
    // The adapter cannot distinguish clean EOF from a backend failure, so
    // reading past the end must fail loudly with UnexpectedEof, never
    // return Ok(0): a genuine backend error must not look like a clean EOF.
    let data: [u8; 3] = [0xAA, 0xBB, 0xCC];
    let mut r = BufBitReader::<BE, _>::new(MemWordReader::new(&data));
    let mut buf = [0u8; 8];
    assert_eq!(std::io::Read::read(&mut r, &mut buf).unwrap(), 3);
    let err = std::io::Read::read(&mut r, &mut buf).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);

    let mut r = BufBitReader::<LE, _>::new(MemWordReader::new(&data));
    let mut buf = [0u8; 8];
    assert_eq!(std::io::Read::read(&mut r, &mut buf).unwrap(), 3);
    let err = std::io::Read::read(&mut r, &mut buf).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);

    // Ok(0) still means "empty buffer", as the Read contract requires, even
    // on an exhausted stream.
    let mut r = BufBitReader::<BE, _>::new(MemWordReader::new(&data));
    assert_eq!(std::io::Read::read(&mut r, &mut []).unwrap(), 0);
    let mut buf = [0u8; 8];
    assert_eq!(std::io::Read::read(&mut r, &mut buf).unwrap(), 3);
    assert_eq!(std::io::Read::read(&mut r, &mut []).unwrap(), 0);
}

#[test]
fn read_exact_and_bounded_read_to_end_work_on_finite_input() {
    let data: [u8; 3] = [0xAA, 0xBB, 0xCC];
    // read_exact of the exact length succeeds...
    let mut r = BufBitReader::<LE, _>::new(MemWordReader::new(&data));
    let mut buf = [0u8; 3];
    std::io::Read::read_exact(&mut r, &mut buf).unwrap();
    assert_eq!(&buf, &data);
    // ...and read_to_end works through a bounded `take`, the documented way
    // to read a known-length stream to its end.
    let r = BufBitReader::<BE, _>::new(MemWordReader::new(&data));
    let mut out = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(r, 3), &mut out).unwrap();
    assert_eq!(out, data);
}
