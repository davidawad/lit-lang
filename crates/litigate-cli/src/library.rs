//! The library compiled into one engine catalog: the packs of every
//! package under `--lib` and the scenarios of the matters under
//! `--cases`, which `litigate mcp` serves as resources and litigate-wasm
//! answers its requests over. There is no JSON input: the catalog is
//! always compiled from `.lit` and `.case` source.
use crate::cmd::run::compile;
use crate::out::Failure;
use litigate_engine::api::Catalog;
use litigate_sema::pkg::source::{Disk, Source};
use std::path::Path;

/// The catalog of the `lib` packages (default `lit`) and the `cases`
/// matters (default `cases`, when it exists) on the disk, compiled.
///
/// # Errors
/// Usage or I/O failures compiling the library.
pub fn library_catalog(lib: &[String], cases: &[String]) -> Result<Catalog, Failure> {
    compiled(&Disk, lib, cases)
}

/// The catalog of the `lib` packages (default `lit`) and the `cases`
/// matters (default `cases`, when it exists) in `src`, compiled. A matter
/// with errors is left out.
///
/// # Errors
/// Usage or I/O failures.
pub fn compiled(src: &dyn Source, lib: &[String], cases: &[String]) -> Result<Catalog, Failure> {
    let mut paths: Vec<String> = crate::ws::library_dirs_in(src, lib)
        .iter()
        .map(|d| d.to_string_lossy().into_owned())
        .collect();
    if cases.is_empty() {
        paths.extend(src.is_dir(Path::new("cases")).then(|| "cases".to_owned()));
    } else {
        paths.extend(cases.iter().cloned());
    }
    Ok(compile(src, &paths, true, lib)?.catalog)
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_sema::pkg::source::MemoryTree;

    const FIXTURE: &str = "tests/fixtures/cov-cmds";

    /// The coverage fixture's `lit/` and `cases/` at the root of a tree.
    fn tree() -> MemoryTree {
        let files = [
            "lit/cov/Lit.toml",
            "lit/cov/cov.lit",
            "cases/Lit.lock",
            "cases/cov.case",
        ]
        .map(|p| {
            let text = std::fs::read_to_string(format!("{FIXTURE}/{p}")).unwrap_or_default();
            (p.to_owned(), text)
        });
        MemoryTree::new(files)
    }

    #[test]
    fn the_cases_directory_is_the_default_scenario_library() {
        let catalog = compiled(&tree(), &[], &[]).unwrap();
        assert_eq!(catalog.scenarios.len(), 1);
        assert_eq!(catalog.packs.len(), 1);
    }

    #[test]
    fn without_a_cases_directory_there_is_no_scenario_library() {
        let lib = [format!("{FIXTURE}/lit")];
        let catalog = library_catalog(&lib, &[]).unwrap();
        assert!(catalog.scenarios.is_empty());
        assert_eq!(catalog.packs.len(), 1);
        let cases = [format!("{FIXTURE}/cases")];
        let catalog = library_catalog(&lib, &cases).unwrap();
        assert_eq!(catalog.scenarios.len(), 1);
    }
}
