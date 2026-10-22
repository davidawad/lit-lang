//! Prints the CST and diagnostics of a `.lit`/`.case` file.
use litigate_syntax::{parse, FileKind};
fn main() {
    let path = std::env::args().nth(1).unwrap_or_default();
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let kind = FileKind::from_path(std::path::Path::new(&path)).unwrap_or(FileKind::Lit);
    let parsed = parse(&text, kind);
    if std::env::args().nth(2).is_some() {
        println!("{}", parsed.debug_tree());
    }
    for d in parsed.diagnostics(&litigate_base::FileId(path.clone())) {
        println!(
            "{}:{}:{} {} {}",
            path,
            d.span.line,
            d.span.col,
            d.code.as_str(),
            d.message
        );
    }
    assert_eq!(parsed.syntax().to_string(), text);
}
