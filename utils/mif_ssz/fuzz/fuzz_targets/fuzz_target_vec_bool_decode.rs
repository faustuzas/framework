#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate mif_ssz;

use mif_ssz::{decode, DecodeError};

// Fuzz ssz_decode()
fuzz_target!(|data: &[u8]| {
    let _result: Result<Vec<bool>, DecodeError> = decode(data);
});
