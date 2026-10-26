//! The `litigate` toolchain (DESIGN §8, SPEC §9.3) as a library: every
//! subcommand is a function returning one envelope
//! `{ok, op, result, diagnostics, provenance, elapsed_ms}`, so the binary,
//! tests, and other front ends share one implementation.
//!
//! Exit codes mirror the engine's: `0` ok, `2` the input ran and failed
//! (an error diagnostic, or an engine `ok: false` from `run` or an op
//! subcommand), `1` usage or I/O failure. `litigate engine` is the
//! engine's own raw interface ([`engine`]) and prints its response, not an
//! envelope.
//!
//! ```
//! let printed = litigate_cli::execute(["litigate", "--format", "json", "explain", "L3004"]);
//! assert_eq!(printed.code, 0);
//! let envelope: serde_json::Value = serde_json::from_str(&printed.stdout).map_err(|e| e.to_string())?;
//! assert_eq!(envelope["result"]["code"], "L3004");
//! # Ok::<(), String>(())
//! ```
pub mod cache;
pub mod cli;
pub mod clock;
mod cmd;
pub mod describe;
pub mod engine;
pub mod filing;
pub mod hirx;
pub mod ics;
pub mod inline_tests;
pub mod matter;
pub mod out;
pub mod prov;
pub mod queries;
mod render;
pub mod report;
pub mod span;
#[cfg(test)]
mod testkit;
pub mod timeline;
pub mod verdict;
pub mod ws;

pub use cmd::check::{check_session, check_session_with, Checked};
pub use litigate_base::LitigateRun;

use clap::Parser;
use cli::{Cli, Format};
use litigate_sema::pkg::source::{Disk, MemoryTree, Source};
use out::{Failure, Printed};
use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Parses `args` (including the program name), runs the command, and
/// returns what to print. Never panics on bad input.
pub fn execute<I, T>(args: I) -> Printed
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    execute_on(args, std::io::stdout().is_terminal())
}

/// The commands [`execute_in`] runs: the ones that only read their inputs
/// and write build outputs, both through the source.
pub const IN_MEMORY_COMMANDS: [&str; 3] = ["check", "build", "worlds"];

/// [`execute`] over `src` instead of the disk, for hosts with no
/// filesystem (WebAssembly): paths resolve in `src`, `build` writes into
/// it, and output is never a terminal's. Only [`IN_MEMORY_COMMANDS`] run;
/// any other command is a usage failure (exit 1).
pub fn execute_in<I, T>(src: &dyn Source, args: I) -> Printed
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = args.into_iter().map(Into::into).collect();
    execute_with(args, Format::Json, false, src, true)
}

/// [`execute_in`] over a [`MemoryTree`] of `(path, text)` files; `args`
/// leave out the program name. The run carries what `build` wrote. The
/// vendored L0 texts the binary embeds (`sources/*.txt`) are in the tree
/// unless `files` gives its own, so a lock pins the same texts it pins
/// over the repository (SPEC §4.2).
pub fn execute_in_memory(
    files: impl IntoIterator<Item = (String, String)>,
    args: &[String],
) -> LitigateRun {
    let mut files: std::collections::BTreeMap<String, String> = files.into_iter().collect();
    for (path, text) in &litigate_engine::cite::SourceCorpus::embedded_ref().files {
        files.entry(path.clone()).or_insert_with(|| text.clone());
    }
    let tree = MemoryTree::new(files);
    let printed = execute_in(
        &tree,
        std::iter::once("litigate").chain(args.iter().map(String::as_str)),
    );
    LitigateRun {
        code: printed.code,
        stdout: printed.stdout,
        stderr: printed.stderr,
        files: tree.written(),
    }
}

/// [`execute`] for a standard output that is (`tty`) or is not a
/// terminal: a terminal defaults to colored human output.
fn execute_on<I, T>(args: I, tty: bool) -> Printed
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = args.into_iter().map(Into::into).collect();
    let format = if tty { Format::Human } else { Format::Json };
    execute_with(args, format, tty, &Disk, false)
}

/// Parses and runs `args` over `src`, in `format` unless `--format` says
/// otherwise and in color when `color`; `in_memory` limits the commands to
/// [`IN_MEMORY_COMMANDS`].
fn execute_with(
    args: Vec<OsString>,
    format: Format,
    color: bool,
    src: &dyn Source,
    in_memory: bool,
) -> Printed {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let rendered = error.render().to_string();
            let informational = matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            );
            return Printed {
                stdout: if informational {
                    rendered.clone()
                } else {
                    String::new()
                },
                stderr: if informational {
                    String::new()
                } else {
                    rendered
                },
                code: u8::from(!informational),
            };
        }
    };
    let format = cli.format.unwrap_or(format);
    let op = cli.command.name();
    if in_memory && !IN_MEMORY_COMMANDS.contains(&op) {
        let message = format!(
            "only {} run over an in-memory source",
            IN_MEMORY_COMMANDS.join(" and ")
        );
        return out::print(op, Err(Failure::new(message)), format, color, 0.0);
    }
    if let cli::Command::Engine { args } = cli.command {
        return engine::execute(
            std::iter::once("litigate engine".to_owned()).chain(args),
            engine::Builtin::Compiled,
        );
    }
    let started = Stopwatch::start();
    let outcome = cmd::dispatch(cli.command, src);
    out::print(op, outcome, format, color, started.elapsed_ms())
}

/// Wall-clock timing. `std::time::Instant` panics on
/// `wasm32-unknown-unknown` (no clock), so there it reports 0.
struct Stopwatch(
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))] std::time::Instant,
);

impl Stopwatch {
    fn start() -> Self {
        Self(
            #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
            std::time::Instant::now(),
        )
    }

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    fn elapsed_ms(&self) -> f64 {
        self.0.elapsed().as_secs_f64() * 1000.0
    }

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    #[allow(clippy::unused_self)] // no clock on this target
    fn elapsed_ms(&self) -> f64 {
        0.0
    }
}

/// The binary's `main`: [`execute`], print, exit.
pub fn main<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    if args.get(1).is_some_and(|arg| arg == "lsp") {
        return lsp(&args[2..]);
    }
    let printed = execute(args);
    // A closed pipe cannot be reported anywhere; the exit code still is.
    let _ = std::io::stdout().write_all(printed.stdout.as_bytes());
    let _ = std::io::stderr().write_all(printed.stderr.as_bytes());
    ExitCode::from(printed.code)
}

/// `litigate lsp --stdio` (SPEC §9.3): the `litigate-lsp` server, which
/// links this library (so it cannot be linked into it), run from beside
/// this binary (else from `PATH`) with stdio inherited.
fn lsp(args: &[OsString]) -> ExitCode {
    if args != ["--stdio"] {
        eprintln!("usage: litigate lsp --stdio");
        return ExitCode::from(1);
    }
    ExitCode::from(serve(&lsp_program(std::env::current_exe().ok().as_deref())))
}

/// The `litigate-lsp` beside `exe` when it exists, else the bare name
/// (looked up on `PATH`).
fn lsp_program(exe: Option<&Path>) -> PathBuf {
    let name = format!("litigate-lsp{}", std::env::consts::EXE_SUFFIX);
    exe.and_then(Path::parent)
        .map(|dir| dir.join(&name))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(&name))
}

/// Runs `program --stdio` with stdio inherited; its exit code (1 when it
/// cannot start or dies by a signal).
fn serve(program: &Path) -> u8 {
    match Command::new(program).arg("--stdio").status() {
        Ok(status) => status
            .code()
            .and_then(|c| u8::try_from(c).ok())
            .unwrap_or(1),
        Err(error) => {
            eprintln!(
                "litigate lsp: cannot run `{}`: {error} (install it with `cargo install --path crates/litigate-lsp`)",
                program.display()
            );
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every call through one instantiation of the generic `execute_on`.
    fn run(args: &[&str], tty: bool) -> Printed {
        execute_on(args.to_vec(), tty)
    }

    #[test]
    fn a_terminal_defaults_to_human_output() {
        let printed = run(&["litigate", "explain", "L3004"], true);
        assert_eq!(printed.code, 0);
        assert!(
            printed.stdout.starts_with("litigate explain: ok"),
            "{}",
            printed.stdout
        );
        assert!(printed.stderr.is_empty());
    }

    #[test]
    fn a_pipe_defaults_to_json() {
        let printed = run(&["litigate", "explain", "L3004"], false);
        let envelope: serde_json::Value = serde_json::from_str(&printed.stdout).unwrap_or_default();
        assert_eq!(envelope["result"]["code"], "L3004");
    }

    #[test]
    fn the_engine_is_never_dispatched_as_an_envelope_command() {
        let engine = cli::Command::Engine { args: Vec::new() };
        assert!(cmd::dispatch(engine, &Disk).is_err());
        let printed = run(&["litigate", "engine", "--version"], false);
        assert_eq!(printed.code, 0);
        assert!(
            printed.stdout.starts_with("litigate engine "),
            "{}",
            printed.stdout
        );
    }

    #[test]
    fn help_goes_to_stdout_and_usage_errors_to_stderr() {
        let help = run(&["litigate", "--help"], false);
        assert_eq!((help.code, help.stderr.as_str()), (0, ""));
        assert!(help.stdout.contains("Usage"), "{}", help.stdout);
        let bad = run(&["litigate", "frobnicate"], false);
        assert_eq!((bad.code, bad.stdout.as_str()), (1, ""));
        assert!(bad.stderr.contains("frobnicate"), "{}", bad.stderr);
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("litigate-lib-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_server_beside_the_binary_wins_over_path() {
        let dir = scratch("beside");
        let bare = PathBuf::from(format!("litigate-lsp{}", std::env::consts::EXE_SUFFIX));
        let exe = dir.join("litigate");
        assert_eq!(lsp_program(Some(&exe)), bare);
        assert_eq!(lsp_program(None), bare);
        assert_eq!(lsp_program(Some(Path::new("/"))), bare);
        std::fs::write(dir.join(&bare), "").unwrap();
        assert_eq!(lsp_program(Some(&exe)), dir.join(&bare));
    }

    #[cfg(unix)]
    fn script(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn serve_passes_the_server_exit_code_through() {
        let dir = scratch("serve");
        let exits = dir.join("exits");
        script(&exits, "exit 7");
        assert_eq!(serve(&exits), 7);
        let killed = dir.join("killed");
        script(&killed, "kill -9 $$");
        assert_eq!(serve(&killed), 1);
        assert_eq!(serve(&dir.join("missing")), 1);
    }
}
