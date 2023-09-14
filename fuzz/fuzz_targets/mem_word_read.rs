#![no_main]
use dsi_bitstream::fuzz::mem_word_read::*;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: FuzzCase| { harness(data) });
