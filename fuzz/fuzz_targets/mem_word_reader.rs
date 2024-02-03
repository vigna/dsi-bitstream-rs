#![no_main]
use dsi_bitstream::fuzz::mem_word_reader::*;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: FuzzCase| harness(data));
