//! Process helpers for the editor tests: finding programs and reporting
//! skips.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The program named by environment variable `env` (e.g. `EMACS`), else
/// `name`, resolved against `PATH`. `None` when it is not installed.
#[must_use]
pub fn find_program(env: &str, name: &str) -> Option<PathBuf> {
    let wanted = std::env::var_os(env).unwrap_or_else(|| name.into());
    let wanted = PathBuf::from(wanted);
    if wanted.components().count() > 1 {
        return wanted.is_file().then_some(wanted);
    }
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join(&wanted))
            .find(|candidate| candidate.is_file())
    })
}

/// Report a skipped test: one `SKIP <test>: <reason>` line on stderr and,
/// when `TEST_SKIP_LOG` names a file, appended to it.
pub fn skip(test: &str, reason: &str) {
    let line = format!("SKIP {test}: {reason}\n");
    eprint!("{line}");
    if let Some(log) = std::env::var_os("TEST_SKIP_LOG") {
        let appended = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log)
            .and_then(|mut file| file.write_all(line.as_bytes()));
        if let Err(error) = appended {
            eprintln!("warning: could not append to TEST_SKIP_LOG: {error}");
        }
    }
}

/// Run `command`, echo its stdout/stderr, and fail with both when it exits
/// non-zero.
///
/// # Errors
/// When the command cannot start or exits unsuccessfully.
pub fn run(command: &mut Command) -> anyhow::Result<Output> {
    let output = command
        .output()
        .map_err(|error| anyhow::anyhow!("cannot run {command:?}: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    print!("{stdout}");
    eprint!("{stderr}");
    anyhow::ensure!(
        output.status.success(),
        "{command:?} failed with {}",
        output.status
    );
    Ok(output)
}

/// A fresh, empty scratch directory `base/name` (removed first if present).
///
/// # Errors
/// When the directory cannot be removed or created.
pub fn scratch_dir(base: &Path, name: &str) -> std::io::Result<PathBuf> {
    let dir = base.join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
