//! Compiles the generated C parser.

fn main() {
    let src_dir = std::path::Path::new("src");
    let mut build = cc::Build::new();
    build.std("c11").include(src_dir).warnings(false);
    #[cfg(target_env = "msvc")]
    build.flag("-utf-8");
    let parser = src_dir.join("parser.c");
    println!("cargo:rerun-if-changed={}", parser.display());
    build.file(parser);
    println!("cargo:rerun-if-changed=src/tree_sitter/parser.h");
    build.compile("tree-sitter-litigate");
}
