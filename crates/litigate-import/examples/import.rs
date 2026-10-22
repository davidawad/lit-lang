//! Imports legacy JSON files into `.lit`/`.case` source under an output
//! directory: `cargo run -p litigate-import --example import -- OUT JSON...`.
//! Prints the diagnostics as JSON and exits 2 when the import fails.
use litigate_import::{import, Input};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(out) = args.next() else {
        eprintln!("usage: import OUT_DIR JSON...");
        return ExitCode::from(1);
    };
    let mut inputs = Vec::new();
    for path in args {
        match std::fs::read_to_string(&path) {
            Ok(text) => inputs.push(Input::new(path, text)),
            Err(error) => {
                eprintln!("{path}: {error}");
                return ExitCode::from(1);
            }
        }
    }
    match import(&inputs) {
        Ok(imported) => {
            for file in imported.files {
                let path = std::path::Path::new(&out).join(&file.path);
                let written = path
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&path, file.text));
                if let Err(error) = written {
                    eprintln!("{}: {error}", path.display());
                    return ExitCode::from(1);
                }
                println!("{} <- {}", file.path, file.from);
            }
            ExitCode::SUCCESS
        }
        Err(diagnostics) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&diagnostics).unwrap_or_default()
            );
            ExitCode::from(2)
        }
    }
}
