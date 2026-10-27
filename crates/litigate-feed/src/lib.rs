//! The opinion feed (lg-8fdj.1): polls `CourtListener` for new opinions from
//! every court a promoted package models and for opinions citing a
//! precedent layer, and files one bead per opinion with its URL, court,
//! date and the elements it may edit (an eyecite-style cite match plus an
//! element-name match). Opinions are deduped by `CourtListener` opinion id
//! against a state file, and requests are paced to `CourtListener`'s rate
//! limits. Public law only: it reads published opinions and the packages'
//! own manifests and builds.
//!
//! `litigate-feed --repo . --state feed-state.json --file` is the daily
//! run (maintenance/opinion-feed/README.md); without `--file` it prints the
//! beads it would file and records nothing.

pub mod bead;
pub mod candidates;
pub mod cite;
pub mod courtlistener;
pub mod date;
pub mod http;
pub mod inventory;
pub mod state;

use bead::{Draft, Filer};
use cite::Extractor;
use courtlistener::{Client, Hit, Transport};
use inventory::{Element, Inventory};
use serde::Serialize;
use state::State;
use std::collections::{BTreeMap, BTreeSet};

/// One poll's bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The earliest filing date polled (`YYYY-MM-DD`).
    pub since: String,
    /// Today: recorded as the last poll when every bead was filed.
    pub today: String,
    /// The most opinions read (and beads filed) in one run; the rest wait
    /// for the next run.
    pub max_opinions: usize,
}

/// What a poll did.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    /// The earliest filing date polled.
    pub since: String,
    /// `CourtListener` court id -> the packages modeling it.
    pub courts: BTreeMap<String, Vec<String>>,
    /// Precedent package -> the `CourtListener` opinion ids it resolved to.
    pub precedents: BTreeMap<String, Vec<u64>>,
    /// Beads filed (or, with no filer, that would be filed).
    pub beads: Vec<Draft>,
    /// Whether `beads` were filed.
    pub filed: bool,
    /// Clusters skipped as already filed.
    pub already_filed: Vec<u64>,
    /// Opinions left for the next run by `max_opinions`.
    pub deferred: usize,
    /// Requests sent to `CourtListener`.
    pub requests: u32,
    /// Problems that did not stop the poll.
    pub warnings: Vec<String>,
    /// Failures after which an opinion was not filed.
    pub errors: Vec<String>,
}

/// The default first day to poll: the day before the last complete poll
/// (an overlap the dedupe absorbs), else yesterday.
#[must_use]
pub fn default_since(state: &State, today: &str) -> String {
    let from = state.last_poll.as_deref().unwrap_or(today);
    date::add_days(from, -1).unwrap_or_else(|| from.to_owned())
}

/// The packages whose elements a poll of `inventory` can match: the
/// watched packages and their dependencies.
#[must_use]
pub fn watched(inventory: &Inventory) -> BTreeSet<String> {
    let roots: Vec<&str> = inventory
        .courts
        .values()
        .flatten()
        .map(String::as_str)
        .chain(inventory.precedents.iter().map(|p| p.package.as_str()))
        .collect();
    inventory.closure(roots)
}

/// An opinion cluster a poll found, why, and the packages it concerns.
struct Found {
    hit: Hit,
    why: Vec<String>,
    packages: BTreeSet<String>,
}

/// Polls `CourtListener` through `client` for opinions filed since
/// `options.since` from the courts in `inventory` or citing its
/// precedents, skips those `state` has filed, and drafts a bead for each
/// with its candidate `elements`; with a `filer`, files each and records
/// it in `state` (and `options.today` as the last poll when nothing failed
/// or was deferred).
///
/// # Errors
/// A failed search (before anything is filed).
pub fn poll<T: Transport>(
    inventory: &Inventory,
    elements: &BTreeMap<String, Vec<Element>>,
    client: &mut Client<T>,
    state: &mut State,
    mut filer: Option<&mut dyn Filer>,
    options: &Options,
) -> Result<Report, String> {
    let mut report = Report {
        since: options.since.clone(),
        courts: inventory.courts.clone(),
        filed: filer.is_some(),
        ..Report::default()
    };
    let found = search(inventory, client, &options.since, &mut report)?;
    let extractor = Extractor::new(&inventory.prefix_groups());
    for Found { hit, why, packages } in found.into_values() {
        if state.seen_any(&hit.opinions) {
            report.already_filed.push(hit.cluster);
            continue;
        }
        if report.beads.len() >= options.max_opinions {
            report.deferred += 1;
            continue;
        }
        let text = match hit
            .opinions
            .iter()
            .map(|id| client.opinion_text(*id))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(texts) => texts.join("\n"),
            Err(e) => {
                report.errors.push(format!("cluster {}: {e}", hit.cluster));
                continue;
            }
        };
        let scope = inventory.closure(packages.iter().map(String::as_str));
        let pool: Vec<&Element> = scope
            .iter()
            .filter_map(|p| elements.get(p))
            .flatten()
            .collect();
        let draft = Draft::new(&hit, why, candidates::candidates(&text, &pool, &extractor));
        if let Some(filer) = filer.as_deref_mut() {
            if let Err(e) = filer.file(&draft) {
                report.errors.push(format!("cluster {}: {e}", hit.cluster));
                continue;
            }
            for id in &draft.opinions {
                state.seen.insert(*id, draft.title.clone());
            }
        }
        report.beads.push(draft);
    }
    if report.filed && report.errors.is_empty() && report.deferred == 0 {
        state.last_poll = Some(options.today.clone());
    }
    report.requests = client.requests;
    Ok(report)
}

/// The clusters filed since `since` from the inventory's courts or citing
/// its precedents, by date and cluster id, a cluster found twice once with
/// both reasons; each precedent's opinion ids go into `report`.
fn search<T: Transport>(
    inventory: &Inventory,
    client: &mut Client<T>,
    since: &str,
    report: &mut Report,
) -> Result<BTreeMap<(String, u64), Found>, String> {
    let mut found: BTreeMap<(String, u64), Found> = BTreeMap::new();
    let mut add = |hit: Hit, why: String, packages: &[String]| {
        let entry = found
            .entry((hit.date.clone(), hit.cluster))
            .or_insert_with(|| Found {
                hit,
                why: Vec::new(),
                packages: BTreeSet::new(),
            });
        entry.why.push(why);
        entry.packages.extend(packages.iter().cloned());
    };
    if !inventory.courts.is_empty() {
        let ids: Vec<&str> = inventory.courts.keys().map(String::as_str).collect();
        for hit in client.search(&[("court", &ids.join(" "))], since)? {
            if let Some(packages) = inventory.courts.get(&hit.court) {
                let why = format!("from {}, modeled by {}", hit.court, packages.join(", "));
                add(hit, why, packages);
            }
        }
    }
    for precedent in &inventory.precedents {
        let opinions = match (precedent.cluster, &precedent.citation) {
            (Some(cluster), _) => client.cluster_opinions(cluster)?,
            (None, Some(citation)) => client.lookup(citation)?,
            (None, None) => Vec::new(),
        };
        report
            .precedents
            .insert(precedent.package.clone(), opinions.clone());
        if opinions.is_empty() {
            report.warnings.push(format!(
                "precedent `{}`: no CourtListener opinion (give a source URL on \
                 courtlistener.com or a designation it resolves)",
                precedent.package
            ));
            continue;
        }
        let ids: Vec<String> = opinions.iter().map(u64::to_string).collect();
        let query = format!("cites:({})", ids.join(" OR "));
        for hit in client.search(&[("q", &query)], since)? {
            let why = format!("cites precedent layer {}", precedent.package);
            add(hit, why, std::slice::from_ref(&precedent.package));
        }
    }
    Ok(found)
}
