#![no_main]
use dsi_bitstream::fuzz::codes::*;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: FuzzCase| harness(data));
