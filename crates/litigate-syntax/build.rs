//! Generate `SyntaxKind`'s keyword variants from `keywords.txt` (see
//! `src/keyword_table.rs`): `$OUT_DIR/keywords.rs` defines the
//! `syntax_kinds_with_keywords!` macro `src/syntax_kind.rs` invokes.

#[allow(dead_code)]
#[path = "src/keyword_table.rs"]
mod keyword_table;

fn main() {
    println!("cargo:rerun-if-changed=keywords.txt");
    let rows = match keyword_table::parse(keyword_table::TABLE) {
        Ok(rows) => rows,
        Err(message) => {
            eprintln!("keywords.txt {message}");
            std::process::exit(1);
        }
    };
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default());
    if let Err(error) = std::fs::write(out.join("keywords.rs"), keyword_table::rust_macro(&rows)) {
        eprintln!("writing keywords.rs: {error}");
        std::process::exit(1);
    }
}
