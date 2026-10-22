//! `litigate build`.
use crate::cli::Dates;
use crate::out::{Failure, Outcome};
use litigate_base::Profile;
use std::path::Path;

/// Writes `text` to `path` atomically (temp file, then rename).
///
/// # Errors
/// I/O failures.
pub fn write_atomic(path: &Path, text: &str) -> Result<(), Failure> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)
            .map_err(|e| Failure::new(format!("{}: {e}", dir.display())))?;
    }
    let tmp = path.with_extension("litigate-tmp");
    std::fs::write(&tmp, text).map_err(|e| Failure::new(format!("{}: {e}", tmp.display())))?;
    std::fs::rename(&tmp, path).map_err(|e| Failure::new(format!("{}: {e}", path.display())))
}

pub fn build(_p: &[String], _o: &str, _pr: Profile, _d: &Dates, _l: &[String]) -> Outcome {
    Err(Failure::new("x"))
}
