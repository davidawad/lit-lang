//! Matching docket entries to declared event forms (DESIGN §10, the
//! docket matcher): the NLP boundary between a court's free-text docket
//! and a typed `.case` event. It is lexical and fail-closed: an entry is
//! *supported* only when one event form has every word of its name in
//! the entry's text (the most specific, when the words of every other
//! that does are among its own), every argument is settled by the text
//! (an enum argument by one variant, chosen the same way), and exactly
//! one party of the matter follows `by`, `on` or `upon`. Anything else
//! stays unresolved, with the candidates and their confidence kept for
//! review. A match gives an event no meaning: replay
//! of the typed event, under the package's rules, decides what it does.
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

/// One docket entry as the source wrote it, with the fields read from it.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The entry verbatim.
    pub source: Value,
    /// `entry_number`: the docket sequence number, when the court gave one.
    pub number: Option<u64>,
    /// `date_filed`.
    pub date: Option<String>,
    /// `description`.
    pub text: String,
}

/// The entries of a docket: a JSON array of entries, or an object holding
/// one under `docket_entries` or `results` (a RECAP / CourtListener
/// export). Each entry is an object; only `entry_number`, `date_filed`
/// and `description` are read, and the rest is kept as written.
///
/// # Errors
/// When the document holds no entry array, or an entry is not an object.
pub fn entries(doc: &Value) -> Result<Vec<Entry>, String> {
    let list = doc
        .as_array()
        .or_else(|| doc.get("docket_entries").and_then(Value::as_array))
        .or_else(|| doc.get("results").and_then(Value::as_array))
        .ok_or(
            "a docket is an array of entries, or an object with `docket_entries` or `results`",
        )?;
    list.iter()
        .enumerate()
        .map(|(i, e)| {
            let o = e
                .as_object()
                .ok_or_else(|| format!("docket entry {i} is not an object"))?;
            Ok(Entry {
                source: e.clone(),
                number: o.get("entry_number").and_then(Value::as_u64),
                date: o
                    .get("date_filed")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                text: o
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
            })
        })
        .collect()
}

/// Words that carry no matching weight.
const STOP: [&str; 16] = [
    "a", "an", "and", "as", "at", "by", "for", "in", "of", "on", "or", "re", "the", "to", "upon",
    "with",
];

/// The lowercase words of `text`: runs of letters and digits, with a
/// `camelCase` or `snake_case` name split at its humps and underscores.
fn words(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut prev_lower = false;
    for c in text.chars() {
        if !c.is_alphanumeric() {
            out.extend((!word.is_empty()).then(|| std::mem::take(&mut word)));
            prev_lower = false;
            continue;
        }
        if c.is_uppercase() && prev_lower {
            out.push(std::mem::take(&mut word));
        }
        prev_lower = c.is_lowercase() || c.is_numeric();
        word.extend(c.to_lowercase());
    }
    out.extend((!word.is_empty()).then_some(word));
    out
}

/// `word` without one inflectional suffix (`-ing`, `-ed`, `-es`, `-s`)
/// when at least three letters remain: `Answered` and `answer` compare equal.
fn stem(word: &str) -> String {
    ["ing", "ed", "es", "s"]
        .iter()
        .find_map(|s| {
            word.strip_suffix(s)
                .filter(|rest| rest.chars().count() >= 3)
        })
        .unwrap_or(word)
        .to_owned()
}

/// The stems of `text`'s words that carry weight.
fn stems(text: &str) -> BTreeSet<String> {
    words(text)
        .iter()
        .filter(|w| !STOP.contains(&w.as_str()))
        .map(|w| stem(w))
        .collect()
}

/// A parameter of an event form, as far as matching sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Param {
    /// A closed enum: its variants.
    Enum(Vec<String>),
    /// Any other type (named for the reason): no entry text settles it.
    Other(String),
}

/// A declared event form a matter may record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form {
    /// The constructor as the matter writes it (`Served`, `pack::Served`).
    pub name: String,
    /// Its parameters in order: `(name, kind)`.
    pub params: Vec<(String, Param)>,
}

/// A party of the matter: its id and the actor it plays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Party {
    /// The local id the event line names.
    pub id: String,
    /// The actor key (`Defendant`).
    pub actor: String,
}

/// How well one event form matches an entry.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Candidate {
    /// The constructor.
    pub event: String,
    /// The share of the name's words found in the entry (`matched /
    /// (matched + missing)`).
    pub confidence: f64,
    /// Name words found.
    pub matched: Vec<String>,
    /// Name words not found.
    pub missing: Vec<String>,
    /// The complete call, when the text settles every argument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<String>,
    /// Why the text does not settle the call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsettled: Option<String>,
}

/// What an entry supports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// One complete event: `(call, mode, party)`.
    Supported(String, &'static str, String),
    /// No form has every word of its name in the entry.
    Unmatched,
    /// More than one form, variant or party fits.
    Ambiguous(String),
    /// One form fits, but the text leaves part of the event open.
    Incomplete(String),
}

/// The one of `sets` whose words include every other's (and no other's
/// equal them): the most specific of several that fit.
fn most_specific(sets: &[BTreeSet<String>]) -> Option<usize> {
    let mut found = (0..sets.len()).filter(|&i| sets.iter().all(|o| o.is_subset(&sets[i])));
    match (found.next(), found.next()) {
        (Some(i), None) => Some(i),
        _ => None,
    }
}

/// The variant of `variants` the text settles: of those whose every word
/// is present, the [`most_specific`]; else whether several were present.
fn variant<'v>(present: &BTreeSet<String>, variants: &'v [String]) -> Result<&'v str, bool> {
    let (found, words): (Vec<&String>, Vec<BTreeSet<String>>) = variants
        .iter()
        .map(|v| (v, stems(v)))
        .filter(|(_, w)| !w.is_empty() && w.is_subset(present))
        .unzip();
    most_specific(&words)
        .map(|i| found[i].as_str())
        .ok_or(!found.is_empty())
}

/// The candidate `form` makes against an entry with stems `present`.
fn candidate(form: &Form, present: &BTreeSet<String>) -> Candidate {
    let local = form.name.rsplit("::").next().unwrap_or(&form.name);
    let name = stems(local);
    let (matched, missing): (Vec<String>, Vec<String>) =
        name.into_iter().partition(|w| present.contains(w));
    let total = matched.len() + missing.len();
    #[allow(clippy::cast_precision_loss)] // word counts are tiny
    let confidence = if total == 0 {
        0.0
    } else {
        matched.len() as f64 / total as f64
    };
    let args: Result<Vec<&str>, String> = form
        .params
        .iter()
        .map(|(p, kind)| match kind {
            Param::Enum(vs) => variant(present, vs).map_err(|several| {
                if several {
                    format!(
                        "ambiguous argument `{p}`: more than one of {} fits",
                        vs.join(", ")
                    )
                } else {
                    format!("argument `{p}`: none of {} is named", vs.join(", "))
                }
            }),
            Param::Other(t) => Err(format!(
                "argument `{p}`: a `{t}` is not read from docket text"
            )),
        })
        .collect();
    let (call, unsettled) = match args {
        Ok(a) => (Some(format!("{}({})", form.name, a.join(", "))), None),
        Err(e) => (None, Some(e)),
    };
    Candidate {
        event: form.name.clone(),
        confidence,
        matched,
        missing,
        call,
        unsettled,
    }
}

/// The parties named right after `by` (acting) or `on` / `upon` (acted
/// upon), by id or actor: `(mode, party id)`.
fn named_parties(text: &str, parties: &[Party]) -> BTreeSet<(&'static str, String)> {
    let w = words(text);
    let mut out = BTreeSet::new();
    for (i, word) in w.iter().enumerate() {
        let mode = match word.as_str() {
            "by" => "by",
            "on" | "upon" => "on",
            _ => continue,
        };
        let after = &w[i + 1..];
        for p in parties {
            let names = [words(&p.id), words(&p.actor)];
            if names.iter().any(|n| !n.is_empty() && after.starts_with(n)) {
                out.insert((mode, p.id.clone()));
            }
        }
    }
    out
}

/// The candidates for an entry with text `text` among `forms`, best first
/// (ties in declaration order; forms sharing no word are left out), and
/// what the entry supports.
#[must_use]
pub fn judge(text: &str, forms: &[Form], parties: &[Party]) -> (Vec<Candidate>, Verdict) {
    let present = stems(text);
    let mut candidates: Vec<Candidate> = forms
        .iter()
        .map(|f| candidate(f, &present))
        .filter(|c| !c.matched.is_empty())
        .collect();
    candidates.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    let full: Vec<&Candidate> = candidates.iter().filter(|c| c.missing.is_empty()).collect();
    let words: Vec<BTreeSet<String>> = full
        .iter()
        .map(|c| c.matched.iter().cloned().collect())
        .collect();
    let verdict = match (full.as_slice(), most_specific(&words)) {
        ([], _) => Verdict::Unmatched,
        (_, Some(i)) => match (&full[i].call, &full[i].unsettled) {
            (Some(call), _) => {
                let named = named_parties(text, parties);
                match named.iter().collect::<Vec<_>>().as_slice() {
                    [(mode, party)] => Verdict::Supported(call.clone(), mode, party.clone()),
                    [] => Verdict::Incomplete(
                        "no party of the matter is named after `by`, `on` or `upon`".to_owned(),
                    ),
                    several => Verdict::Ambiguous(format!(
                        "more than one party fits: {}",
                        several
                            .iter()
                            .map(|(m, p)| format!("{m} {p}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                }
            }
            (None, Some(why)) if why.starts_with("ambiguous") => Verdict::Ambiguous(why.clone()),
            (None, why) => Verdict::Incomplete(why.clone().unwrap_or_default()),
        },
        (several, None) => Verdict::Ambiguous(format!(
            "more than one event form fits: {}",
            several
                .iter()
                .map(|c| c.event.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    };
    (candidates, verdict)
}

#[cfg(test)]
mod tests;
