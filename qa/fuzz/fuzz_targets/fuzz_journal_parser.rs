#![no_main]

use libfuzzer_sys::fuzz_target;
use sentry_fuzz::parse_journal_export_entry;

fuzz_target!(|data: &[u8]| {
    // Parser must never panic, enter infinite loop, or exceed memory budgets
    let mut cursor = 0;
    while cursor < data.len() {
        match parse_journal_export_entry(&data[cursor..]) {
            Ok(Some((_entry, consumed))) => {
                if consumed == 0 {
                    break;
                }
                cursor += consumed;
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }
});
