#![no_main]

use libfuzzer_sys::fuzz_target;
use sentry_fuzz::parse_psi_record;

fuzz_target!(|data: &[u8]| {
    // Convert arbitrary bytes to UTF-8 lossy and test PSI metric parser
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = parse_psi_record(text);
    }
});
