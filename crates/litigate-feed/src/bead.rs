//! The bead filed for an opinion, and filing it with `br create`.

use crate::candidates::Candidate;
use crate::courtlistener::Hit;
use serde::Serialize;
use std::fmt::Write as _;
use std::process::Command;

/// The label every opinion-feed bead carries.
pub const LABEL: &str = "opinion-feed";

/// A bead to file for one opinion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Draft {
    /// The bead title.
    pub title: String,
    /// The bead description: the opinion and its candidate elements.
    pub description: String,
    /// Its labels.
    pub labels: Vec<String>,
    /// `courtlistener:opinion:<lead opinion id>`: the dedupe key the store
    /// (or the dispatcher applying `br create` intents) dedupes on.
    pub external_ref: String,
    /// The opinion ids it covers (the feed's dedupe key).
    pub opinions: Vec<u64>,
    /// The opinion URL.
    pub url: String,
    /// The `CourtListener` court id.
    pub court: String,
    /// The date filed.
    pub date: String,
    /// Why the feed picked it up.
    pub why: Vec<String>,
    /// The elements it may edit.
    pub candidates: Vec<Candidate>,
}

impl Draft {
    /// The bead for `hit`, found for `why`, with `candidates`.
    #[must_use]
    pub fn new(hit: &Hit, why: Vec<String>, candidates: Vec<Candidate>) -> Self {
        let lead = hit.opinions.first().copied().unwrap_or_default();
        let mut description = format!(
            "New opinion for the modeled law (opinion feed, lg-8fdj.1).\n\n\
             Opinion: {}\nCase: {}\nCourt: {}\nDate filed: {}\nCourtListener cluster {}, opinions {}\n",
            hit.url,
            hit.case_name,
            hit.court,
            hit.date,
            hit.cluster,
            join(&hit.opinions),
        );
        if !hit.citations.is_empty() {
            let _ = writeln!(description, "Citations: {}", hit.citations.join("; "));
        }
        description.push_str("\nWhy:\n");
        for reason in &why {
            let _ = writeln!(description, "- {reason}");
        }
        description.push_str("\nCandidate elements (eyecite-style cite match plus element-name match; review each):\n");
        if candidates.is_empty() {
            description
                .push_str("- none matched: read the opinion for edits the matcher cannot see\n");
        }
        for c in &candidates {
            let _ = writeln!(
                description,
                "- {}::{} \"{}\"{}: {}",
                c.package,
                c.element,
                c.label,
                c.cite
                    .as_ref()
                    .map(|s| format!(" [{s}]"))
                    .unwrap_or_default(),
                c.reasons.join("; ")
            );
        }
        let mut packages: Vec<&str> = candidates.iter().map(|c| c.package.as_str()).collect();
        packages.sort_unstable();
        packages.dedup();
        let mut labels = vec![LABEL.to_owned(), format!("court:{}", hit.court)];
        labels.extend(packages.iter().map(|p| format!("pkg:{p}")));
        Self {
            title: format!(
                "Opinion feed: {} ({}, {}): {} candidate element{}",
                hit.case_name,
                hit.court,
                hit.date,
                candidates.len(),
                if candidates.len() == 1 { "" } else { "s" }
            ),
            description,
            labels,
            external_ref: external_ref(lead),
            opinions: hit.opinions.clone(),
            url: hit.url.clone(),
            court: hit.court.clone(),
            date: hit.date.clone(),
            why,
            candidates,
        }
    }

    /// The `br` arguments filing this bead.
    #[must_use]
    pub fn br_args(&self) -> Vec<String> {
        [
            "create",
            &self.title,
            "--type",
            "task",
            "--priority",
            "3",
            "--labels",
            &self.labels.join(","),
            "--external-ref",
            &self.external_ref,
            "--description",
            &self.description,
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect()
    }
}

/// The external ref of the bead for `CourtListener` opinion `id`.
#[must_use]
pub fn external_ref(id: u64) -> String {
    format!("courtlistener:opinion:{id}")
}

fn join(ids: &[u64]) -> String {
    ids.iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Files a bead.
pub trait Filer {
    /// Files `draft`.
    ///
    /// # Errors
    /// The bead was not filed.
    fn file(&mut self, draft: &Draft) -> Result<(), String>;
}

/// Files with `<program> create ... --external-ref courtlistener:opinion:<id>`
/// (`br`). In a pod `br` is the shim recording each call as a `BR_INTENT`
/// line for the dispatcher to apply, so filing needs no writable store.
pub struct Br {
    /// The `br` program.
    pub program: String,
}

impl Filer for Br {
    fn file(&mut self, draft: &Draft) -> Result<(), String> {
        let output = Command::new(&self.program)
            .args(draft.br_args())
            .output()
            .map_err(|e| format!("cannot run `{}`: {e}", self.program))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "`{} create` failed ({}): {}",
                self.program,
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
}
