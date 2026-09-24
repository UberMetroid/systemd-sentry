#![no_main]

use libfuzzer_sys::fuzz_target;
use sentry_fuzz::decode_dbus_message_header;

fuzz_target!(|data: &[u8]| {
    // Fuzz binary header decoder: validates safe parsing without panics
    let _ = decode_dbus_message_header(data);
});
