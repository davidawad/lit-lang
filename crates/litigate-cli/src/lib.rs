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
pub mod docket;
pub mod engine;
pub mod filing;
pub mod hirx;
pub mod ics;
pub mod inline_tests;
pub mod matter;
pub mod mcp;
pub mod out;
pub mod prov;
pub mod queries;
mod render;
pub mod report;
pub mod span;
#[cfg(test)]
mod testkit;
pub mod timeline;
pub mod today;
pub mod verdict;
pub mod ws;

/// What `--version` prints after the program name: the package version,
/// the git commit built from, and whether the tree was `clean` or `dirty`
/// (`build.rs`; `unknown` outside a git checkout), e.g.
/// `0.6.0 (commit 06a430f…, clean)`, since installs go by sha.
pub const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (commit ",
    env!("LITIGATE_GIT_COMMIT"),
    ", ",
    env!("LITIGATE_GIT_TREE"),
    ")"
);

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
    execute_on(args, std::io::stdout().is_terminal(), None)
}

/// The commands [`execute_in`] runs, besides the op subcommands
/// (`solve`, `chain`, ...): the ones that only read their inputs and write
/// build outputs, both through the source.
pub const IN_MEMORY_COMMANDS: [&str; 5] = ["check", "build", "worlds", "run", "engine"];

/// Whether [`execute_in`] runs `command`: one of [`IN_MEMORY_COMMANDS`] or
/// an op subcommand.
fn runs_in_memory(command: &cli::Command) -> bool {
    matches!(command, cli::Command::Op(_)) || IN_MEMORY_COMMANDS.contains(&command.name())
}

/// [`execute`] over `src` instead of the disk, for hosts with no
/// filesystem (WebAssembly): paths resolve in `src`, `build` writes into
/// it, and output is never a terminal's. Only [`IN_MEMORY_COMMANDS`] and
/// the op subcommands run, compiling the matter or package in `src` (and
/// `engine` its `lit/` and `cases/`); any other command is a usage
/// failure (exit 1). `check` judges deadlines at `today` unless told a
/// date, as the binary does at its own today ([`main`]); `None` judges
/// them at each matter's `as_of`.
pub fn execute_in<I, T>(src: &dyn Source, args: I, today: Option<&str>) -> Printed
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = args.into_iter().map(Into::into).collect();
    execute_with(args, Format::Json, false, src, true, today)
}

/// [`execute_in`] over a [`MemoryTree`] of `(path, text)` files; `args`
/// leave out the program name. The run carries what `build` wrote. The
/// vendored L0 texts the binary embeds (`sources/*.txt`) are in the tree
/// unless `files` gives its own, so a lock pins the same texts it pins
/// over the repository (SPEC §4.2). `today` is [`execute_in`]'s.
pub fn execute_in_memory(
    files: impl IntoIterator<Item = (String, String)>,
    args: &[String],
    today: Option<&str>,
) -> LitigateRun {
    let tree = memory_tree(files);
    let printed = execute_in(
        &tree,
        std::iter::once("litigate").chain(args.iter().map(String::as_str)),
        today,
    );
    LitigateRun {
        code: printed.code,
        stdout: printed.stdout,
        stderr: printed.stderr,
        files: tree.written(),
    }
}

/// A [`MemoryTree`] of `(path, text)` files with the vendored L0 texts
/// (`sources/*.txt`) the binary embeds filled in wherever `files` gives
/// none, as [`execute_in_memory`] runs over.
#[must_use]
pub fn memory_tree(files: impl IntoIterator<Item = (String, String)>) -> MemoryTree {
    let mut files: std::collections::BTreeMap<String, String> = files.into_iter().collect();
    for (path, text) in &litigate_engine::cite::SourceCorpus::embedded_ref().files {
        files.entry(path.clone()).or_insert_with(|| text.clone());
    }
    MemoryTree::new(files)
}

/// [`execute`] for a standard output that is (`tty`) or is not a
/// terminal: a terminal defaults to colored human output. `check` and
/// `timeline` judge deadlines at `today` unless told a date
/// ([`cli::Command::default_today`]).
fn execute_on<I, T>(args: I, tty: bool, today: Option<&str>) -> Printed
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = args.into_iter().map(Into::into).collect();
    let format = if tty { Format::Human } else { Format::Json };
    execute_with(args, format, tty, &Disk, false, today)
}

/// Parses and runs `args` over `src`, in `format` unless `--format` says
/// otherwise and in color when `color`; `in_memory` limits the commands to
/// those [`execute_in`] runs; `check` and `timeline` judge deadlines at
/// `today` unless told a date.
fn execute_with(
    args: Vec<OsString>,
    format: Format,
    color: bool,
    src: &dyn Source,
    in_memory: bool,
    today: Option<&str>,
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
    let mut command = match cli.command {
        cli::Invocation::Engine { args } => {
            return engine::execute_in(
                src,
                std::iter::once("litigate engine".to_owned()).chain(args),
                engine::Builtin::Compiled,
            )
        }
        cli::Invocation::Envelope(command) => command,
    };
    if let Some(today) = today {
        command.default_today(today);
    }
    let format = cli.format.unwrap_or(format);
    let op = command.name();
    if in_memory && !runs_in_memory(&command) {
        let message = format!(
            "only {} and the op subcommands run over an in-memory source",
            IN_MEMORY_COMMANDS.join(", ")
        );
        return out::print(op, Err(Failure::new(message)), format, color, 0.0);
    }
    let started = Stopwatch::start();
    let outcome = cmd::dispatch(command, src);
    out::print(op, outcome, format, color, started.elapsed_ms())
}

/// Wall-clock timing. `std::time::Instant` panics on
/// `wasm32-unknown-unknown` (no clock), so there it reports 0.
pub(crate) struct Stopwatch(
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

/// The binary's `main`: [`execute`], print, exit; `check` and `timeline`
/// judge deadlines at [`today::today`] unless told a date.
pub fn main<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    match args.get(1).and_then(|arg| arg.to_str()) {
        Some("lsp") => return lsp(&args[2..]),
        Some("mcp") => return mcp::main(&args[2..]),
        _ => {}
    }
    let tty = std::io::stdout().is_terminal();
    let printed = execute_on(args, tty, today::today().as_deref());
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
        execute_on(args.to_vec(), tty, None)
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
    fn version_names_the_commit_and_whether_the_tree_was_dirty() {
        let commit = env!("LITIGATE_GIT_COMMIT");
        let hex = commit.bytes().filter(u8::is_ascii_hexdigit).count();
        assert!(
            matches!((commit, commit.len(), hex), ("unknown", ..) | (_, 40, 40)),
            "{commit}"
        );
        assert!(["clean", "dirty", "unknown"].contains(&env!("LITIGATE_GIT_TREE")));
        for args in [
            &["litigate", "--version"][..],
            &["litigate", "engine", "--version"],
        ] {
            let printed = run(args, false);
            assert_eq!(printed.code, 0);
            assert!(
                printed.stdout.contains(&format!(" (commit {commit}, ")),
                "{}",
                printed.stdout
            );
            assert!(printed
                .stdout
                .trim_end()
                .ends_with(concat!(env!("LITIGATE_GIT_TREE"), ")")));
        }
    }

    #[test]
    fn the_engine_prints_its_own_response_not_an_envelope() {
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
