//! Fuzz the two strict TOML readers used to identify and pin packages.
#![no_main]

use libfuzzer_sys::fuzz_target;
use litigate_sema::pkg::{lock::LockFile, manifest::parse_manifest};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(lock) = LockFile::parse(text) {
        // A normalized lock must survive its canonical serialization.
        let reparsed = LockFile::parse(&lock.render()).expect("rendered lock parses");
        assert_eq!(reparsed, LockFile::new(lock.packages));
    }
    let _ = parse_manifest(text);
});
