#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    patholog_fuzz::run_cli_read_only_bytes(data);
});
