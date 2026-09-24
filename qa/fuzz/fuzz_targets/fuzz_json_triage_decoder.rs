#![no_main]

use libfuzzer_sys::fuzz_target;
use sentry_fuzz::parse_and_validate_diagnostic_json;

fuzz_target!(|data: &[u8]| {
    // Tests markdown stripping, brace isolation, and schema parsing
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = parse_and_validate_diagnostic_json(text);
    }
});
