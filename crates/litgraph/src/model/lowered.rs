// SPDX-License-Identifier: GPL-3.0-or-later
//! The `lowered` header of a compiler-generated document (pack schema v4,
//! links and scenario schema v2, clock schema v4): which package versions
//! it was built from, at which dates, the digest of that world, and every
//! hole the source left open. A document without it is hand-authored.
//!
//! A hole is an explicit `?` in the source: the field it stands for is
//! absent from the document (or, for a cite, holds the hole's reason
//! text), and `holes` names it, so a dev build's gaps are never mistaken
//! for authored law.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::canonical;
use crate::error::{Error, Result};

/// One package the document was built from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    /// Package name.
    pub name: String,
    /// Exact version.
    pub version: String,
    /// `sha256:<hex>` source digest of the package.
    pub digest: String,
}

/// An explicit hole: the source wrote `?` where this document has no
/// authored value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hole {
    /// JSON pointer (RFC 6901) of the field in this document.
    pub pointer: String,
    /// The `?("reason")` text, when given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl Hole {
    /// Whether the hole stands for a number the engine computes with: a
    /// node's or edge's `probability`, `cost`, `hours`, `payoff`,
    /// `payoffByFlag`, `duration` or `attrs`, or an instance's `stake`
    /// (the field right below `/nodes/i`, `/edges/i`, `/links/i` or
    /// `/instances/name`). A cite or note hole is not.
    #[must_use]
    pub fn numeric(&self) -> bool {
        const NUMERIC: [&str; 8] = [
            "probability",
            "cost",
            "hours",
            "payoff",
            "payoffByFlag",
            "duration",
            "attrs",
            "stake",
        ];
        self.pointer
            .split('/')
            .nth(3)
            .is_some_and(|field| NUMERIC.contains(&field))
    }
}

/// Provenance of a lowered document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Lowered {
    /// Every package of the world the document was built in, by name.
    pub pins: Vec<Pin>,
    /// Law-in-force date versions were selected at.
    pub as_of: String,
    /// Corpus date versions were selected among (absent: the latest).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_at: Option<String>,
    /// `sha256:<hex>` over the JCS text (RFC 8785) of
    /// `{"asOf", "knownAt", "pins"}`: the world's identity.
    pub world_digest: String,
    /// Explicit holes, by pointer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holes: Vec<Hole>,
}

impl Lowered {
    /// The header for `pins` at `(as_of, known_at)`, with pins and holes
    /// sorted and the world digest computed.
    #[must_use]
    pub fn new(
        mut pins: Vec<Pin>,
        as_of: String,
        known_at: Option<String>,
        mut holes: Vec<Hole>,
    ) -> Self {
        pins.sort();
        pins.dedup();
        holes.sort();
        holes.dedup();
        let world_digest = world_digest(&pins, &as_of, known_at.as_deref());
        Self {
            pins,
            as_of,
            known_at,
            world_digest,
            holes,
        }
    }

    /// Checks the header against `doc`, the whole document it heads: pins
    /// sorted and unique by name, the world digest recomputed, and every
    /// hole's parent present in `doc`.
    ///
    /// # Errors
    /// `Error::Parse` naming what does not hold.
    pub fn check(&self, doc: &Value) -> Result<()> {
        self.problem(doc).map_or(Ok(()), |m| Err(Error::Parse(m)))
    }

    /// What [`Lowered::check`] rejects, if anything.
    fn problem(&self, doc: &Value) -> Option<String> {
        if self.pins.windows(2).any(|w| w[0].name >= w[1].name) {
            return Some("lowered.pins must be sorted by name, each package once".into());
        }
        let want = world_digest(&self.pins, &self.as_of, self.known_at.as_deref());
        if self.world_digest != want {
            return Some(format!(
                "lowered.worldDigest is {}, but its pins and dates digest to {want}",
                self.world_digest
            ));
        }
        let h = self.holes.iter().find(|h| !inside(doc, &h.pointer))?;
        Some(format!(
            "lowered hole `{}` points outside the document",
            h.pointer
        ))
    }
}

/// Whether `pointer` addresses a place in `doc` a hole can be: its first
/// member present, every array index on the way in bounds, and nothing
/// below a scalar. Object members past the first may be absent, since a
/// hole is usually the absence of its field.
fn inside(doc: &Value, pointer: &str) -> bool {
    let Some(rest) = pointer.strip_prefix('/') else {
        return false;
    };
    let mut at = doc;
    for (depth, token) in rest.split('/').enumerate() {
        let token = token.replace("~1", "/").replace("~0", "~");
        let next = match at {
            Value::Array(items) => token.parse::<usize>().ok().and_then(|i| items.get(i)),
            Value::Object(map) => match map.get(&token) {
                Some(v) => Some(v),
                None => return depth > 0,
            },
            _ => None,
        };
        match next {
            Some(v) => at = v,
            None => return false,
        }
    }
    true
}

/// The world digest of `pins` at `(as_of, known_at)`.
#[must_use]
pub fn world_digest(pins: &[Pin], as_of: &str, known_at: Option<&str>) -> String {
    let mut world = json!({"asOf": as_of, "pins": pins});
    if let Some(k) = known_at {
        world["knownAt"] = Value::String(k.to_owned());
    }
    canonical::digest(&world)
}

/// Checks the `lowered` header of a document read at `version`: allowed
/// only from `since`, and consistent with the document.
///
/// # Errors
/// `Error::Parse` naming the document `what`.
pub(crate) fn check_header<T: Serialize>(
    what: &str,
    version: u32,
    since: u32,
    lowered: Option<&Lowered>,
    doc: &T,
) -> Result<()> {
    let Some(lowered) = lowered else {
        return Ok(());
    };
    if version < since {
        return Err(Error::Parse(format!(
            "{what}: `lowered` needs schemaVersion {since}, found {version}"
        )));
    }
    let value = serde_json::to_value(doc).map_err(|e| Error::Parse(e.to_string()))?;
    lowered
        .problem(&value)
        .map_or(Ok(()), |m| Err(Error::Parse(format!("{what}: {m}"))))
}

/// A JSON pointer reference token (RFC 6901 §4): `~` is `~0`, `/` is `~1`.
#[must_use]
pub fn pointer_token(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(name: &str) -> Pin {
        Pin {
            name: name.into(),
            version: "1.0.0".into(),
            digest: "sha256:00".into(),
        }
    }

    fn hole(pointer: &str) -> Hole {
        Hole {
            pointer: pointer.into(),
            reason: None,
        }
    }

    #[test]
    fn new_sorts_and_digests_the_world() {
        let a = Lowered::new(
            vec![pin("b"), pin("a"), pin("a")],
            "2025-01-01".into(),
            None,
            vec![],
        );
        assert_eq!(a.pins, [pin("a"), pin("b")]);
        let b = Lowered::new(vec![pin("a"), pin("b")], "2025-01-01".into(), None, vec![]);
        assert_eq!(a.world_digest, b.world_digest);
        let known = Lowered::new(
            b.pins.clone(),
            b.as_of.clone(),
            Some("2024-12-01".into()),
            vec![],
        );
        assert_ne!(known.world_digest, b.world_digest);
        assert!(a.world_digest.starts_with("sha256:"));
    }

    #[test]
    fn check_rejects_unsorted_pins_stale_digests_and_stray_holes() {
        let doc = json!({"edges": [{"to": "x"}], "lowered": {}});
        let ok = Lowered::new(
            vec![pin("a"), pin("b")],
            "2025-01-01".into(),
            None,
            vec![hole("/edges/0/probability")],
        );
        assert!(ok.check(&doc).is_ok());
        let mut unsorted = ok.clone();
        unsorted.pins.reverse();
        assert!(unsorted
            .check(&doc)
            .is_err_and(|e| e.to_string().contains("sorted")));
        let mut stale = ok.clone();
        stale.as_of = "2026-01-01".into();
        assert!(stale
            .check(&doc)
            .is_err_and(|e| e.to_string().contains("digest to")));
        let absent = Lowered::new(
            vec![],
            "2025-01-01".into(),
            None,
            vec![hole("/edges/0/duration/min")],
        );
        assert!(absent.check(&doc).is_ok());
        for stray in [
            "/edges/1/probability",
            "",
            "edges",
            "/nodes/0",
            "/edges/x",
            "/edges/0/to/x",
        ] {
            let mut bad = ok.clone();
            bad.holes = vec![hole(stray)];
            assert!(bad
                .check(&doc)
                .is_err_and(|e| e.to_string().contains("outside")));
        }
    }

    #[test]
    fn the_header_needs_its_schema_version() {
        let lowered = Lowered::new(vec![], "2025-01-01".into(), None, vec![]);
        let doc = json!({});
        assert!(check_header("pack p", 3, 4, None, &doc).is_ok());
        assert!(check_header("pack p", 3, 4, Some(&lowered), &doc)
            .is_err_and(|e| e.to_string().contains("needs schemaVersion 4")));
        assert!(check_header("pack p", 4, 4, Some(&lowered), &doc).is_ok());
        let mut bad = lowered;
        bad.world_digest = "sha256:00".into();
        assert!(check_header("pack p", 4, 4, Some(&bad), &doc)
            .is_err_and(|e| e.to_string().contains("pack p: lowered.worldDigest")));
    }

    #[test]
    fn numeric_holes_are_the_fields_the_engine_computes_with() {
        for (pointer, numeric) in [
            ("/edges/0/probability", true),
            ("/nodes/2/payoffByFlag/won", true),
            ("/instances/a@b/stake", true),
            ("/edges/0/authority", false),
            ("/nodes/0/cite", false),
            ("/edges", false),
        ] {
            assert_eq!(hole(pointer).numeric(), numeric, "{pointer}");
        }
    }

    #[test]
    fn pointer_tokens_escape() {
        assert_eq!(pointer_token("a/b~c"), "a~1b~0c");
    }
}
