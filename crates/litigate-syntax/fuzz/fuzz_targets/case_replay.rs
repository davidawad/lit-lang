//! Fuzz `.case` parsing and its lossless replay input representation.
#![no_main]

use libfuzzer_sys::fuzz_target;
use litigate_syntax::{parse, FileKind};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let parsed = parse(text, FileKind::Case);
    assert_eq!(parsed.syntax().to_string(), text);
    // Reading the case AST walks the replay inputs (ordered events, facts,
    // and analysis) and must remain total even while the parser recovers.
    let _ = litigate_syntax::ast::CaseFile::cast(parsed.syntax().clone());
});
