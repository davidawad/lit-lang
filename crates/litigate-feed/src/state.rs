//! What the feed has already filed: the opinion ids it filed a bead for
//! (the dedupe key) and the day of its last complete poll.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// The feed's state file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// The day (`YYYY-MM-DD`) of the last poll that filed every bead it
    /// found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_poll: Option<String>,
    /// Opinion id -> the bead title filed for it.
    #[serde(default)]
    pub seen: BTreeMap<u64, String>,
}

impl State {
    /// The state at `path`; empty when there is no file yet.
    ///
    /// # Errors
    /// An unreadable or malformed file.
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }

    /// Writes the state to `path`.
    ///
    /// # Errors
    /// An unwritable path.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = serde_json::to_string_pretty(self).unwrap_or_default();
        std::fs::write(path, text + "\n").map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Whether any of `opinions` was filed already.
    #[must_use]
    pub fn seen_any(&self, opinions: &[u64]) -> bool {
        opinions.iter().any(|id| self.seen.contains_key(id))
    }
}
