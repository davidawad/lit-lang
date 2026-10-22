//! Checks package directories and prints every diagnostic:
//! `cargo run -p litigate-sema --example check -- REPO_ROOT DIR...`.
//! Exits 2 when any package has an error or produced no HIR.
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1).map(PathBuf::from);
    let Some(root) = args.next() else {
        eprintln!("usage: check REPO_ROOT DIR...");
        return ExitCode::from(1);
    };
    let dirs: Vec<PathBuf> = args.collect();
    let (db, ws) = match litigate_sema::load(&root, &dirs, None) {
        Ok(loaded) => loaded,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };
    let mut failed = false;
    for check in litigate_sema::check_workspace(&db, ws) {
        for d in &check.diagnostics {
            failed |= d.severity == litigate_base::Severity::Error;
            println!(
                "{}:{}:{} {} {}",
                d.span.file.0,
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            );
        }
        failed |= check.package.is_none();
    }
    if failed {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}
