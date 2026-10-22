//! Formats a `.lit`/`.case` file to stdout.
use litigate_syntax::{format, FileKind};
fn main() {
    let path = std::env::args().nth(1).unwrap_or_default();
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let kind = FileKind::from_path(std::path::Path::new(&path)).unwrap_or(FileKind::Lit);
    match format(&text, kind) {
        Ok(out) => print!("{out}"),
        Err(e) => eprintln!("{e}"),
    }
}
