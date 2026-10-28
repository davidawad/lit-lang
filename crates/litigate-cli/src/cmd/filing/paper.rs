//! `lint-filing`'s paper check (SPEC §3.7, lg-962.10.2): the filing's
//! counts (`--words`, `--pages`, `--lines`, or a document's words) and
//! the certificates it carries (`--certificate`) against the `limit` of
//! its paper kind in force on the filing date.
//!
//! The paper kind is the event declaration's (`event E() paper k`);
//! `--paper KIND` names it only for an event that declares none, and a
//! flag naming another kind than the declared one is an error. With
//! neither, the kind is a hole. The version checked is the one sema's
//! selection query (SPEC §5.5) chooses over the matter's law on the
//! filing date, so a layer that overrides the base sets the limit, and
//! each violation names the limit, its cite and that layer. A count from
//! a document includes the parts the limit excludes: within a bound it
//! is within, over it is unknown when the limit excludes anything.
use crate::hirx;
use crate::out::Failure;
use litigate_sema::hir::{self, Authored, Counts, Declaration, Limit, Measure, Verdict};
use litigate_sema::pkg::source::Source;
use litigate_sema::precedence::select::chain;
use litigate_sema::temporal::world::{Key, World};
use serde_json::{json, Value};
use std::path::Path;

/// What the filer says of the paper.
#[derive(Debug, Clone, Default)]
pub struct Paper {
    /// Words, without the excluded parts.
    pub words: Option<u64>,
    /// Pages, without the excluded parts.
    pub pages: Option<u64>,
    /// Lines of monospaced type, without the excluded parts.
    pub lines: Option<u64>,
    /// A UTF-8 text of the paper, whose words are counted.
    pub document: Option<String>,
    /// `--paper KIND`: the kind of an event that declares none.
    pub kind: Option<String>,
    /// Certificate kinds the paper carries.
    pub certificates: Vec<String>,
}

impl Paper {
    /// Whether anything was said of the paper: otherwise `lint-filing`
    /// does not check it.
    #[must_use]
    pub fn asked(&self) -> bool {
        self.words.is_some()
            || self.pages.is_some()
            || self.lines.is_some()
            || self.document.is_some()
            || self.kind.is_some()
            || !self.certificates.is_empty()
    }
}

/// The paper's counts, and whether they are gross (a document's, the
/// excluded parts included).
///
/// # Errors
/// `--words` with a document, or a document that cannot be read as text
/// from `src`.
fn counts(src: &dyn Source, paper: &Paper) -> Result<(Counts, bool), Failure> {
    let given = Counts {
        words: paper.words,
        pages: paper.pages,
        lines: paper.lines,
    };
    let Some(path) = &paper.document else {
        return Ok((given, false));
    };
    if paper.words.is_some() {
        return Err(Failure::new(format!(
            "--words and the document {path} both give the word count: give one"
        )));
    }
    let bytes = src
        .read(Path::new(path))
        .map_err(|e| Failure::new(format!("{path}: {e}")))?;
    let text = String::from_utf8(bytes).map_err(|_| {
        Failure::new(format!(
            "{path}: not UTF-8 text; give its counts with --words/--pages/--lines"
        ))
    })?;
    let words = u64::try_from(text.split_whitespace().count()).unwrap_or(u64::MAX);
    Ok((
        Counts {
            words: Some(words),
            ..given
        },
        true,
    ))
}

/// The event constructor `event` (`Name(...)` or `pack::Name(...)`)
/// among `vocabulary` (primary first), with its pack id.
fn event<'p>(
    vocabulary: &[&'p hir::Package],
    event: &str,
) -> Option<(&'p str, &'p hir::EventType)> {
    let callee = event.split('(').next().unwrap_or(event).trim();
    let (pack, name) = callee
        .rsplit_once("::")
        .map_or((None, callee), |(p, n)| (Some(p), n));
    vocabulary
        .iter()
        .filter(|p| pack.is_none_or(|q| p.manifest.pack_id == q))
        .find_map(|p| {
            hirx::declarations(p).find_map(|d| match d {
                Declaration::Event(t) if t.name == name => Some((p.manifest.pack_id.as_str(), t)),
                _ => None,
            })
        })
}

/// The key of paper kind `kind` as written in pack `pack`.
fn key_in(pack: &str, kind: &str) -> Key {
    Key {
        pack: kind.rsplit_once("::").map_or(pack, |(p, _)| p).to_owned(),
        name: hirx::local(kind).to_owned(),
    }
}

/// The key of `--paper KIND`: the one pack of `law` declaring a limit
/// `KIND` (`pack::KIND` picks one).
///
/// # Errors
/// No pack, or more than one, declares it.
fn flagged(law: &[&hir::Package], kind: &str) -> Result<Key, Failure> {
    let pack = kind.rsplit_once("::").map(|(p, _)| p);
    let mut packs: Vec<&str> = law
        .iter()
        .filter(|p| pack.is_none_or(|q| p.manifest.pack_id == q))
        .filter(|p| {
            hirx::declarations(p)
                .any(|d| matches!(d, Declaration::Limit(l) if l.meta.id == hirx::local(kind)))
        })
        .map(|p| p.manifest.pack_id.as_str())
        .collect();
    packs.dedup();
    match packs.as_slice() {
        [one] => Ok(key_in(one, kind)),
        [] => Err(Failure::new(format!(
            "--paper {kind}: no `limit {}` in the matter's law",
            hirx::local(kind)
        ))),
        many => Err(Failure::new(format!(
            "--paper {kind}: declared by {}; qualify it (`pack::{}`)",
            many.join(", "),
            hirx::local(kind)
        ))),
    }
}

/// The paper kind of the filing: the event's, else the flag's, and which
/// said it; `None` when neither names one.
///
/// # Errors
/// The flag names a kind the matter's law lacks, or one other than the
/// event declares.
fn kind(
    law: &[&hir::Package],
    vocabulary: &[&hir::Package],
    filed: &str,
    flag: Option<&str>,
) -> Result<Option<(Key, &'static str)>, Failure> {
    match (declared(vocabulary, filed), flag) {
        (Some(declared), Some(flag)) => {
            let given = flagged(law, flag)?;
            if given == declared {
                Ok(Some((declared, "event")))
            } else {
                Err(Failure::new(format!(
                    "--paper {flag}: `{filed}` files paper `{}::{}`, as its event declares; \
                     --paper names the kind only of an event that declares none",
                    declared.pack, declared.name
                )))
            }
        }
        (Some(declared), None) => Ok(Some((declared, "event"))),
        (None, Some(flag)) => flagged(law, flag).map(|k| Some((k, "flag"))),
        (None, None) => Ok(None),
    }
}

/// A measure's spelling with its count, as a rule writes it.
fn words(measure: Measure, n: u64) -> String {
    format!("{n} {}", measure.as_str())
}

/// ` (cite)`, or nothing without one.
fn cited(cite: Option<String>) -> String {
    cite.map(|c| format!(" ({c})")).unwrap_or_default()
}

/// The paper kind event `filed` (`Name(...)` or `pack::Name(...)`)
/// declares (`event E() paper k`), among `vocabulary` (primary first).
#[must_use]
pub fn declared(vocabulary: &[&hir::Package], filed: &str) -> Option<Key> {
    event(vocabulary, filed).and_then(|(pack, t)| t.paper.as_deref().map(|k| key_in(pack, k)))
}

/// The limit governing a paper kind on a date, and the layers that set
/// it.
#[derive(Debug, Clone)]
pub struct Governing<'a> {
    /// The version selected.
    pub limit: &'a Limit,
    /// The package declaring it: the layer that set the limit.
    pub package: String,
    /// Its pack id.
    pub pack: String,
    /// The packages selection passed through, base first: the base's
    /// package, then each layer whose claim displaced the one before;
    /// the last is [`Governing::package`].
    pub provenance: Vec<String>,
}

impl Governing<'_> {
    /// The layer that set the limit: its package, pack, authority and
    /// version, and the layers it displaced (`provenance`).
    #[must_use]
    pub fn layer(&self) -> Value {
        let meta = &self.limit.meta;
        json!({
            "package": self.package, "pack": self.pack, "authority": meta.authority,
            "version": format!("{}@{}", meta.id, meta.effective.from),
            "provenance": self.provenance,
        })
    }
}

/// The limit of paper kind `key` that SPEC §5.5's selection chooses over
/// `law` on `date` at `known_at`, with the layers that set it.
///
/// # Errors
/// Why none governs (no version in force, unordered claimants), or the
/// winner is not a limit, as a sentence.
pub fn governing<'a>(
    law: &[&'a hir::Package],
    key: &Key,
    date: &str,
    known_at: Option<&str>,
) -> Result<Governing<'a>, String> {
    let name = format!("`{}::{}`", key.pack, key.name);
    let world = World::closure(law, law);
    let chain =
        chain(&world, key, date, known_at).map_err(|why| why.describe(&world, &name, date))?;
    let last = chain[chain.len() - 1];
    let Declaration::Limit(limit) = world.decls[last].decl else {
        return Err(format!("{name} is not a limit"));
    };
    let package_of = |i: usize| world.packs[world.decls[i].pack].name.clone();
    let base = world
        .packs
        .iter()
        .find(|p| p.pack == key.pack)
        .map(|p| p.name.clone());
    let mut provenance: Vec<String> = Vec::new();
    for p in base.into_iter().chain(chain.iter().map(|&i| package_of(i))) {
        if provenance.last() != Some(&p) {
            provenance.push(p);
        }
    }
    Ok(Governing {
        limit,
        package: package_of(last),
        pack: world.pack_of(last).to_owned(),
        provenance,
    })
}

/// `limit` against the counts and certificates: its bounds, certificates,
/// violations (each naming the limit, its cite and `layer`) and holes.
fn judge(
    name: &str,
    limit: &Limit,
    layer: &Value,
    (counts, gross): (Counts, bool),
    carried: &[String],
) -> Value {
    // A document's count includes what the limit excludes, so an
    // exclusion it names is one more part not yet subtracted.
    let mut judged = limit.clone();
    if let Some(ex) = judged
        .excludes
        .as_mut()
        .filter(|e| gross && !e.items.is_empty())
    {
        ex.items.push(Authored::Hole(Some(
            "the document's count includes the excluded parts".to_owned(),
        )));
    }
    let cite = |own: &Option<String>| own.clone().or_else(|| limit.meta.cite.clone());
    let bounds: Vec<Value> = judged
        .bounds
        .iter()
        .map(|b| {
            let max = match &b.max {
                Authored::Known(n) => Some(*n),
                Authored::Hole(_) => None,
            };
            json!({
                "measure": b.measure, "max": max, "count": counts.of(b.measure),
                "verdict": judged.bound(b, &counts), "cite": cite(&b.cite),
            })
        })
        .collect();
    let length = judged.length(&counts);
    let mut violations = Vec::new();
    if length == Some(Verdict::Over) {
        violations.push(json!({
            "limit": name, "requirement": "length", "cite": limit.meta.cite, "layer": layer,
            "message": format!("`{name}`: {}", over(&judged, &counts)),
        }));
    }
    let certificates: Vec<Value> = judged
        .certificates
        .iter()
        .map(|c| {
            let required = judged.requires(c, &counts);
            let has = carried.contains(&c.kind);
            if required == Some(true) && !has {
                violations.push(json!({
                    "limit": name, "requirement": "certificate", "certificate": c.kind,
                    "cite": cite(&c.cite), "layer": layer,
                    "message": format!(
                        "`{name}` requires certificate `{}`{}; pass --certificate {} once the paper carries it",
                        c.kind, cited(cite(&c.cite)), c.kind
                    ),
                }));
            }
            json!({
                "kind": c.kind, "for": c.measures, "cite": cite(&c.cite),
                "required": required, "carried": has,
            })
        })
        .collect();
    json!({
        "limit": name, "cite": limit.meta.cite, "layer": layer,
        "counts": {
            "words": counts.words, "pages": counts.pages, "lines": counts.lines,
            "source": if gross { "document" } else { "flags" },
        },
        "length": length, "bounds": bounds, "certificates": certificates,
        "violations": violations, "holes": holes(name, limit),
    })
}

/// Each bound of `limit` the paper is over, as `N words over M words
/// (cite)`.
fn over(limit: &Limit, counts: &Counts) -> String {
    let over: Vec<String> = limit
        .bounds
        .iter()
        .filter_map(|b| match (&b.max, counts.of(b.measure)) {
            (Authored::Known(max), Some(n)) => Some(format!(
                "{} over {}{}",
                words(b.measure, n),
                words(b.measure, *max),
                cited(b.cite.clone().or_else(|| limit.meta.cite.clone()))
            )),
            _ => None,
        })
        .collect();
    over.join("; ")
}

/// What `limit` (named `name`) leaves unsettled: a bound that is a hole,
/// and an open list of excluded parts.
fn holes(name: &str, limit: &Limit) -> Vec<Value> {
    let bounds = limit.bounds.iter().filter_map(|b| match &b.max {
        Authored::Hole(why) => Some(json!({
            "what": format!("the {} bound of `{name}`", b.measure.as_str()),
            "reason": why,
        })),
        Authored::Known(_) => None,
    });
    let excluded = limit.excludes.iter().filter(|e| e.open()).map(|ex| {
        json!({
            "what": format!("the excluded parts of `{name}`"),
            "reason": ex.items.iter().find_map(|i| match i {
                Authored::Hole(why) => why.clone(),
                Authored::Known(_) => None,
            }),
        })
    });
    bounds.chain(excluded).collect()
}

/// The paper check of filing `filed` on `date` (at `known_at`) under the
/// matter's `law`, its events named as `vocabulary` sees them: `{kind,
/// kind_from, limit, cite, layer, counts, length, bounds, certificates,
/// violations, holes}`.
///
/// # Errors
/// A usage error: `--paper` conflicts with the declared kind or names
/// none, or the counts cannot be read.
pub fn check(
    src: &dyn Source,
    law: &[&hir::Package],
    vocabulary: &[&hir::Package],
    (filed, date, known_at): (&str, &str, Option<&str>),
    paper: &Paper,
) -> Result<Value, Failure> {
    let counts = counts(src, paper)?;
    let empty = |kind: Value, from: Value, hole: String| {
        json!({
            "kind": kind, "kind_from": from, "limit": null, "violations": [],
            "holes": [{"what": "the paper's limit", "reason": hole}],
        })
    };
    let Some((key, from)) = kind(law, vocabulary, filed, paper.kind.as_deref())? else {
        return Ok(empty(
            Value::Null,
            Value::Null,
            format!(
                "the paper kind: `{filed}` declares no `paper` and --paper names none, \
                 so no limit judges it"
            ),
        ));
    };
    let name = format!("{}::{}", key.pack, key.name);
    Ok(match governing(law, &key, date, known_at) {
        Ok(set) => {
            let layer = set.layer();
            let mut out = judge(&name, set.limit, &layer, counts, &paper.certificates);
            out["kind"] = Value::from(key.name);
            out["kind_from"] = Value::from(from);
            out
        }
        Err(why) => empty(Value::from(key.name), Value::from(from), why),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_sema::hir::{CertificateReq, Exclusions, LengthBound};
    use litigate_sema::pkg::source::Disk;

    fn limit(excludes: Vec<Authored<String>>) -> Limit {
        let bound = |measure, max| LengthBound {
            measure,
            max,
            cite: None,
            span: None,
        };
        let mut l = Limit {
            meta: crate::testkit::hir_with("")
                .modules
                .iter()
                .flat_map(|m| m.declarations.iter())
                .find_map(|d| litigate_sema::temporal::world::meta_of(d).cloned())
                .expect("a declaration with metadata"),
            bounds: vec![
                bound(Measure::Pages, Authored::Known(30)),
                bound(Measure::Words, Authored::Known(13_000)),
                bound(Measure::Lines, Authored::Hole(Some("unsettled".into()))),
            ],
            certificates: vec![CertificateReq {
                kind: "compliance".into(),
                measures: vec![Measure::Words, Measure::Lines],
                cite: Some("FRAP 32(g)(1)".into()),
                span: None,
            }],
            excludes: Some(Exclusions {
                items: excludes,
                cite: None,
                span: None,
            }),
            ecf: Vec::new(),
            holes: std::collections::BTreeMap::new(),
        };
        l.meta.cite = Some("FRAP 32(a)(7)".into());
        l
    }

    fn tally(words: Option<u64>, pages: Option<u64>) -> Counts {
        Counts {
            words,
            pages,
            lines: None,
        }
    }

    fn requirements(v: &Value) -> Vec<&str> {
        v["violations"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x["requirement"].as_str()).collect())
            .unwrap_or_default()
    }

    #[test]
    fn a_paper_relying_on_the_word_bound_needs_its_certificate() {
        let l = limit(vec![Authored::Known("cover-page".into())]);
        let layer = json!("base");
        let within = judge(
            "frap::principal-brief",
            &l,
            &layer,
            (tally(Some(12_000), Some(40)), false),
            &[],
        );
        assert_eq!(within["length"], json!("within"));
        assert_eq!(requirements(&within), ["certificate"]);
        assert_eq!(within["violations"][0]["cite"], json!("FRAP 32(g)(1)"));
        assert_eq!(within["violations"][0]["layer"], layer);
        let carried = judge(
            "frap::principal-brief",
            &l,
            &layer,
            (tally(Some(12_000), Some(40)), false),
            &["compliance".to_owned()],
        );
        assert_eq!(requirements(&carried), Vec::<&str>::new());
        // Within the page bound: no type volume relied on, no certificate.
        let short = judge(
            "frap::principal-brief",
            &l,
            &layer,
            (tally(None, Some(20)), false),
            &[],
        );
        assert_eq!(requirements(&short), Vec::<&str>::new());
        assert_eq!(short["certificates"][0]["required"], json!(false));
    }

    #[test]
    fn over_every_bound_is_a_length_violation_unless_a_hole_leaves_it_open() {
        let l = limit(vec![Authored::Known("cover-page".into())]);
        let over = judge(
            "k",
            &l,
            &json!(null),
            (tally(Some(14_000), Some(40)), false),
            &[],
        );
        // The lines bound is a hole: the paper is not over every bound.
        assert_eq!(over["length"], json!("unknown"));
        assert_eq!(over["holes"][0]["reason"], json!("unsettled"));
        let mut settled = l.clone();
        settled.bounds.pop();
        let over = judge(
            "k",
            &settled,
            &json!(null),
            (tally(Some(14_000), Some(40)), false),
            &[],
        );
        assert_eq!(over["length"], json!("over"));
        assert_eq!(requirements(&over), ["length"]);
        assert_eq!(
            over["violations"][0]["message"],
            json!("`k`: 40 pages over 30 pages (FRAP 32(a)(7)); 14000 words over 13000 words (FRAP 32(a)(7))")
        );
        // A document's count includes the excluded parts: over is unknown.
        let gross = judge(
            "k",
            &settled,
            &json!(null),
            (tally(Some(14_000), Some(40)), true),
            &[],
        );
        assert_eq!(gross["length"], json!("unknown"));
        assert_eq!(gross["counts"]["source"], json!("document"));
        // ... unless the limit excludes nothing.
        let mut bare = settled;
        bare.excludes = None;
        let gross = judge(
            "k",
            &bare,
            &json!(null),
            (tally(Some(14_000), Some(40)), true),
            &[],
        );
        assert_eq!(gross["length"], json!("over"));
        // An open exclusion list is a hole too.
        let open = judge(
            "k",
            &limit(vec![Authored::Hole(Some("local rule".into()))]),
            &json!(null),
            (tally(Some(1), None), false),
            &[],
        );
        assert_eq!(open["holes"][1]["reason"], json!("local rule"));
    }

    #[test]
    fn a_document_gives_the_word_count_and_only_that() {
        let dir = std::env::temp_dir().join(format!("lint-filing-paper-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch");
        let doc = dir.join("brief.txt");
        std::fs::write(&doc, "one two\nthree\n\n four").expect("write");
        let paper = |words| Paper {
            words,
            pages: Some(3),
            document: Some(doc.to_string_lossy().into_owned()),
            ..Paper::default()
        };
        assert_eq!(
            counts(&Disk, &paper(None)).map(|(c, g)| (c.words, c.pages, g)),
            Ok((Some(4), Some(3), true))
        );
        assert!(counts(&Disk, &paper(Some(4))).is_err());
        let binary = dir.join("brief.pdf");
        std::fs::write(&binary, [0xff, 0xfe, 0x00]).expect("write");
        let pdf = Paper {
            document: Some(binary.to_string_lossy().into_owned()),
            ..Paper::default()
        };
        assert!(counts(&Disk, &pdf).is_err_and(|e| e.0.contains("not UTF-8")));
        let missing = Paper {
            document: Some(dir.join("none.txt").to_string_lossy().into_owned()),
            ..Paper::default()
        };
        assert!(counts(&Disk, &missing).is_err());
        assert!(!Paper::default().asked());
        assert!(pdf.asked());
    }
}
