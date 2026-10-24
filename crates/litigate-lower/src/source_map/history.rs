//! Amendment-history proofs (SPEC §4.3, §7). A vendored text dated D is
//! the text of a cite whose version took effect at E < D only when the
//! section's vendored amendment history records no amendment in (E, D]:
//! one effective exactly at E made the E text and holds, one at D is part
//! of the vendored text and breaks, one strictly between breaks.
//!
//! A history is a `## versions <heading>` block in a file a source binding
//! pins with `path_sha256`: an `as_of YYYY-MM-DD` line (the date the
//! history was read, which must not precede D), then the versions list as
//! the publisher's versioner returned it (JSON with a `content_versions`
//! array of `{date, amendment_date, issue_date, removed}`). The block
//! itself is the evidence: its path, line and hash go into the cite's
//! provenance. A cite with a section no history proves is a hole (SPEC
//! §7); [`Citations::unproven`] says why, naming the amendment.
use super::{sha256_hex, Citations, L0Section, Pin};
use litgraph::cite::corpus::Section;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The heading prefix of a section's amendment history.
pub const PREFIX: &str = "versions ";

/// The vendored history that proves a cite's text was in force when its
/// version took effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct History {
    /// E: the date the citing version took effect.
    pub effective: String,
    /// Repository-relative file holding the history.
    pub path: String,
    /// One-based line of its `## versions <heading>` line.
    pub line: u32,
    /// SHA-256 (lowercase hex) of the history block's text, trimmed.
    pub sha256: String,
    /// The date the history was read; it records every amendment to then.
    pub as_of: String,
    /// The last recorded version effective on or before E: the version
    /// the cited text is.
    pub version: String,
}

/// One recorded version of a section.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Amendment {
    /// Version date.
    pub date: String,
    /// Effective date of the amendment that made it (else `date`).
    #[serde(default)]
    pub amendment_date: Option<String>,
    /// Date the publisher issued it.
    #[serde(default)]
    pub issue_date: Option<String>,
    /// The amendment removed the section.
    #[serde(default)]
    pub removed: bool,
}

impl Amendment {
    /// The date the amendment took effect.
    #[must_use]
    pub fn effective(&self) -> &str {
        self.amendment_date.as_deref().unwrap_or(&self.date)
    }

    /// The amendment as a diagnostic names it.
    #[must_use]
    pub fn describe(&self) -> String {
        let what = if self.removed { "removed" } else { "amended" };
        let issued = self
            .issue_date
            .as_deref()
            .unwrap_or("on an unrecorded date");
        format!(
            "{what} effective {} (version {}, issued {issued})",
            self.effective(),
            self.date
        )
    }
}

#[derive(Deserialize)]
struct Versions {
    content_versions: Vec<Amendment>,
}

/// Whether a cite's vendored text is its version's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proof {
    /// Nothing to prove: no vendored text, or one undated or not after E.
    Moot,
    /// No amendment in (E, D].
    Holds(History),
    /// No usable history: why.
    Missing(String),
    /// The first amendment effective in (E, D], and how many follow it.
    Broken {
        /// The history read.
        history: History,
        /// The earliest amendment in the window.
        first: Amendment,
        /// How many later amendments are in the window too.
        more: usize,
    },
}

impl Citations<'_> {
    /// Whether `section`'s vendored text (dated by its binding's `as_of`)
    /// is the text in force at `effective`. A section located in no
    /// vendored file has no text to prove.
    #[must_use]
    pub fn prove(&self, section: &L0Section, effective: &str) -> Proof {
        let vendored = section.path.as_ref().and(section.as_of.as_deref());
        let Some(d) = vendored.filter(|d| *d > effective) else {
            return Proof::Moot;
        };
        let heading = format!("{PREFIX}{}", section.heading);
        let verified = |p: &&(String, Section)| {
            self.pins
                .values()
                .any(|b| b.pin == Pin::Verified && b.path.as_ref() == Some(&p.0))
        };
        let found = self
            .corpus
            .and_then(|c| Some((c, c.lookup_all(&heading).iter().find(verified)?)));
        let Some((corpus, (path, block))) = found else {
            return Proof::Missing(format!(
                "no source binding pins a `## {heading}` amendment history"
            ));
        };
        let (first, json) = block.body.split_once('\n').unwrap_or((&block.body, ""));
        let read = first.strip_prefix("as_of ").map(str::trim);
        let Some(as_of) = read.filter(|a| *a >= d) else {
            return Proof::Missing(format!(
                "`## {heading}` in `{path}` does not open with an `as_of` date on or after {d}"
            ));
        };
        let mut versions = match serde_json::from_str::<Versions>(json.trim()) {
            Ok(v) => v.content_versions,
            Err(e) => {
                return Proof::Missing(format!(
                    "`## {heading}` in `{path}` is not a versions list: {e}"
                ))
            }
        };
        versions.sort_by(|a, b| a.effective().cmp(b.effective()));
        let Some(version) = versions.iter().rev().find(|v| v.effective() <= effective) else {
            // The record starts after E: nothing shows the text E made.
            return Proof::Missing(format!(
                "`## {heading}` in `{path}` records no version on or before {effective}"
            ));
        };
        let line = corpus.files[path]
            .lines()
            .position(|l| l.strip_prefix("## ").is_some_and(|h| h.trim() == heading))
            .map_or(0, |i| i + 1);
        let history = History {
            effective: effective.to_owned(),
            path: path.clone(),
            line: u32::try_from(line).unwrap_or(u32::MAX),
            sha256: sha256_hex(&block.body),
            as_of: as_of.to_owned(),
            version: version.date.clone(),
        };
        let mut breaking = versions
            .into_iter()
            .filter(|v| v.effective() > effective && v.effective() <= d);
        match breaking.next() {
            None => Proof::Holds(history),
            Some(first) => Proof::Broken {
                history,
                first,
                more: breaking.count(),
            },
        }
    }

    /// Why `text`, cited by a version in force from `effective`, is a hole:
    /// each of its sections whose vendored text no history proves was the
    /// text then. `None` when every section is proved (or needs no proof).
    #[must_use]
    pub fn unproven(&self, text: &str, effective: &str) -> Option<String> {
        let sections = self
            .map(Some(&text.to_owned()))
            .map_or_else(Vec::new, |m| m.sections);
        let mut why: Vec<String> = sections
            .iter()
            .filter_map(|s| failure(s, &self.prove(s, effective)))
            .collect();
        why.dedup();
        (!why.is_empty()).then(|| {
            format!(
                "`{text}` is not proved the text in force from {effective}: {}",
                why.join("; ")
            )
        })
    }
}

/// Why `proof` of `section` fails, naming the first amendment that breaks
/// it; `None` when it holds or is moot.
fn failure(section: &L0Section, proof: &Proof) -> Option<String> {
    let why = match proof {
        Proof::Moot | Proof::Holds(_) => return None,
        Proof::Missing(why) => why.clone(),
        Proof::Broken {
            history,
            first,
            more,
        } => {
            let later = match more {
                0 => String::new(),
                n => format!(" and {n} later"),
            };
            format!("{}{later}, per `{}`", first.describe(), history.path)
        }
    };
    let source = section.source.as_deref().unwrap_or_default();
    let d = section.as_of.as_deref().unwrap_or_default();
    Some(format!(
        "{} (source `{source}` as of {d}): {why}",
        section.heading
    ))
}
