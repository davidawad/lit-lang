//! Compiles the generated C parser and the hand-written external scanner.

fn main() {
    let src_dir = std::path::Path::new("src");
    let mut build = cc::Build::new();
    build.std("c11").include(src_dir).warnings(false);
    #[cfg(target_env = "msvc")]
    build.flag("-utf-8");
    for file in ["parser.c", "scanner.c"] {
        let path = src_dir.join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        build.file(path);
    }
    println!("cargo:rerun-if-changed=src/tree_sitter/parser.h");
    build.compile("tree-sitter-litigate");
}
