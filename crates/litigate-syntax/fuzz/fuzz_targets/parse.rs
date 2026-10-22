//! Parser fuzz target: parsing must be total and lossless on any UTF-8
//! input, and formatting a cleanly parsed file must be idempotent.
#![no_main]

use libfuzzer_sys::fuzz_target;
use litigate_syntax::{format, parse, FileKind};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    for kind in [FileKind::Lit, FileKind::Case] {
        let parsed = parse(text, kind);
        assert_eq!(parsed.syntax().to_string(), text);
        if parsed.is_ok() {
            let once = format(text, kind).expect("clean parse formats");
            assert_eq!(format(&once, kind).as_deref(), Ok(once.as_str()));
        }
    }
});
