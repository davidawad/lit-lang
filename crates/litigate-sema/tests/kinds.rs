//! The instrument class as a capability index (SPEC §4.1, §5.7; design
//! review 2026-10-23 §2.5): `kind` in `Lit.toml` types a package, each
//! class has an effect row, and a declaration with an effect outside its
//! package's row is a type error (L2010). Every class-to-operation pair
//! the index rejects has a negative test here; the manifest rules that
//! justify a class (instrument, authority, addressees, `hypothetical` as
//! a world modality) are L1004.
mod legal;

use legal::{holdings, pkg, render, workspace, CORE};
use litigate_base::{FileId, LineIndex};
use litigate_sema::hir::{Effect, Kind, Row, Scope};
use litigate_sema::pkg::load::LoadedPackage;
use litigate_sema::pkg::manifest::parse_manifest;
use std::collections::BTreeMap;

const START: &str = "module core.start\nstart z\nstate z: \"Z\"\n";

fn core() -> LoadedPackage {
    pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[("authority.lit", CORE), ("start.lit", START)],
    )
}

/// `fed` (`FederalRule`): a deadline, a default and a counting rule the
/// layer packages act on.
fn fed() -> LoadedPackage {
    let text = "module fed.rules
import core.authority.{LocalRule, StandingOrder, CaseOrder}
start s
event Served()
counting c6 { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
deadline answer-period: 21.days from Served() counted_by c6 effective [2009-12-01, ..) delegable { to LocalRule, op set } delegable { to StandingOrder, op set } delegable { to CaseOrder, op set }
default open when true => true
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
";
    pkg(
        "fed",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[("rules.lit", text)],
    )
}

/// The instrument table every non-base fixture cites.
const INSTRUMENT: &str = "\n[sources.order]\npath = \"sources/order.txt\"\n\n[package.instrument]\nsource = \"order\"\nissuer = \"Demo Court\"\ndate = \"2020-01-01\"\n";

/// A package `order` of class `kind` and `authority` over `fed`, with
/// `extra` instrument keys and `body` declarations.
fn layer(kind: &str, authority: &str, extra: &str, body: &str) -> LoadedPackage {
    let text = format!(
        "module order.rules
import core.authority.{{CaseOrder}}
import fed.rules.{{answer-period, open, c6, cal, Served}}
start l
state l: \"L\"
{body}
"
    );
    let mut p = pkg(
        "order",
        authority,
        "2009-12-01",
        &["core", "fed"],
        &[("rules.lit", &text)],
    );
    p.manifest = p.manifest.replacen(
        "title = \"T\"\n",
        &format!("title = \"T\"\nkind = \"{kind}\"\n"),
        1,
    );
    p.manifest.push_str(INSTRUMENT);
    p.manifest.push_str(extra);
    if kind == "precedent" {
        // Undesignated: it binds nowhere.
        p.manifest.push_str(&holdings(
            &[],
            &["fed::answer-period", "fed::open", "fed::c6", "fed::cal"],
        ));
    }
    p
}

/// The diagnostics but lints of `layer`, keyed by `name`.
fn diagnostics(rows: &[(&str, LoadedPackage)]) -> BTreeMap<String, Vec<String>> {
    rows.iter()
        .map(|(name, layer)| {
            let (db, ws) = workspace(&[core(), fed(), layer.clone()]);
            let found = render(&db, ws)
                .into_iter()
                .filter(|d| !d.contains(" L9"))
                .collect();
            ((*name).to_owned(), found)
        })
        .collect()
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn each_class_rejects_the_operations_outside_its_row() {
    let override_ =
        "deadline local-answer: 30.days from Served() counted_by c6 override answer-period";
    let rows = [
        // Only base law orders authority levels: a local rule that ranks
        // itself above the rules it answers to does not type-check.
        (
            "local-rule ranks",
            layer(
                "local-rule",
                "LocalRule",
                "",
                "authority LocalRule > FederalRule",
            ),
        ),
        // A local rule displaces a federal deadline it is delegated.
        ("local-rule overrides", layer("local-rule", "LocalRule", "", override_)),
        // A standing order gives capabilities; a case order does not.
        (
            "standing-order delegates",
            layer(
                "standing-order",
                "StandingOrder",
                "",
                &format!("{override_} delegable {{ to CaseOrder, op set }}"),
            ),
        ),
        (
            "case-order delegates",
            layer(
                "case-order",
                "CaseOrder",
                "",
                "deadline mine: 7.days from Served() counted_by c6 { extend { by Court, discretion GoodCause } }\ndeadline waivable: 7.days from Served() counted_by c6 { waive { by Court } }\ngrant g on [mine] effective [2020-01-01, 2020-02-01) cite \"Order\" { unlock }",
            ),
        ),
        // Replacing a declaration in time is an amendment's, not a layer's.
        (
            "case-order supersedes",
            layer(
                "case-order",
                "CaseOrder",
                "",
                "deadline mine: 7.days from Served() counted_by c6 supersedes answer-period",
            ),
        ),
        (
            "amendment supersedes",
            layer(
                "amendment",
                "FederalRule",
                "",
                "deadline mine: 7.days from Served() counted_by c6 supersedes answer-period effective [2020-01-01, ..)",
            ),
        ),
        // Persuasive precedent is evidence: it neither prescribes, nor
        // displaces, nor defeats a default, nor states a fact.
        (
            "precedent prescribes and overrides",
            layer("precedent", "CaseOrder", "court = \"ca9\"\n", &format!("{override_}\nexception e rebuts open when true => false\nchance f: \"F\" {{ tags [fact]\n  yes \"Yes\" -> l p 1/2\n  no \"No\" -> l p 1/2\n}}")),
        ),
        // An overruling only retracts: two displacing claims, one report.
        (
            "overruling displaces twice",
            layer(
                "overruling",
                "CaseOrder",
                "court = \"ca9\"\noverrules = [\"p\"]\n",
                "state x: \"X\" override answer-period notwithstanding open",
            ),
        ),
        // Guidance moves its addressees' priors and nothing else.
        (
            "guidance sets priors",
            layer(
                "guidance",
                "CaseOrder",
                "binds = [\"Court\"]\n",
                "chance d: \"D\" by Court {\n  a \"A\" -> l p 1/2\n  b \"B\" -> l p 1/2\n}\nchance o: \"O\" by Defendant {\n  a \"A\" -> l p 1/2\n  b \"B\" -> l p 1/2\n}\ncounting mine { holidays cal }",
            ),
        ),
        // Its own structure is every class's; reviving another package's
        // declaration is versioning it.
        (
            "case-order states its own structure",
            layer(
                "case-order",
                "CaseOrder",
                "",
                "type Amount = Money<usd>\nenum E { a, b }\nevent Ev()\ncalendar mine { new-year: fixed(1, 1) effective [1870-06-28, ..) }\ndefault d when true => 1\nexception own rebuts d when true => 2\nrule duty r {\n  deadline inner: 3.days from Ev() counted_by c6\n}\ndeadline v: 1.days from Ev() counted_by c6 effective [2010-01-01, 2011-01-01)\ndeadline v: 2.days from Ev() counted_by c6 effective [2011-01-01, ..)\ntransition tr amends v@2011-01-01 { otherwise -> v@2011-01-01 }\nstate back: \"Back\" revives answer-period",
            ),
        ),
        // Yielding and a class's own structure need nothing.
        (
            "case-order yields",
            layer(
                "case-order",
                "CaseOrder",
                "",
                "deadline mine: 7.days from Served() counted_by c6 subject_to answer-period\nexception e rebuts open when true => false\ntest \"t\" { expect true == true }",
            ),
        ),
    ];
    insta::assert_debug_snapshot!(diagnostics(&rows));
    // What a class's row permits compiles with no error at all.
    for (name, layer) in rows.into_iter().filter(|(n, _)| {
        matches!(
            *n,
            "local-rule overrides"
                | "standing-order delegates"
                | "amendment supersedes"
                | "case-order yields"
        )
    }) {
        let (db, ws) = workspace(&[core(), fed(), layer]);
        let checks = litigate_sema::check_workspace(&db, ws);
        assert!(
            checks.iter().all(|c| c.package.is_some()),
            "{name}: {:?}",
            render(&db, ws)
        );
    }
}

/// `line:col CODE message` for every report of a manifest `edit`ed from a
/// valid standing order's.
fn manifest(edit: impl Fn(String) -> String) -> Vec<String> {
    let text = edit(layer("standing-order", "StandingOrder", "", "").manifest);
    let index = LineIndex::new(&text);
    let file = FileId("Lit.toml".to_owned());
    parse_manifest(&text)
        .err()
        .unwrap_or_default()
        .iter()
        .map(|r| {
            let d = r.to_diagnostic(&file, &index);
            format!(
                "{}:{} {} {}",
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            )
        })
        .collect()
}

#[test]
fn a_class_is_justified_by_its_instrument() {
    let kind = |k: &'static str| {
        move |t: String| t.replace("kind = \"standing-order\"", &format!("kind = \"{k}\""))
    };
    let instrument = |from: &'static str, to: &'static str| move |t: String| t.replace(from, to);
    let no_instrument =
        |t: String| t[..t.find("\n[package.instrument]").unwrap_or(t.len())].to_owned();
    let rows: Vec<(&str, Vec<String>)> = vec![
        ("valid", manifest(|t| t)),
        ("hypothetical", manifest(kind("hypothetical"))),
        ("unknown", manifest(kind("world"))),
        ("authority", manifest(kind("local-rule"))),
        ("no instrument", manifest(no_instrument)),
        (
            "instrument on base",
            manifest(|t| t.replace("kind = \"standing-order\"\n", "")),
        ),
        ("explicit base with instrument", manifest(kind("base"))),
        (
            "explicit base",
            manifest(|t| no_instrument(kind("base")(t))),
        ),
        (
            "source not vendored",
            manifest(instrument("source = \"order\"", "source = \"elsewhere\"")),
        ),
        (
            "empty issuer, bad date",
            manifest(|t| {
                instrument("date = \"2020-01-01\"", "date = \"2020-13-01\"")(instrument(
                    "issuer = \"Demo Court\"",
                    "issuer = \" \"",
                )(t))
            }),
        ),
        (
            "binds outside guidance",
            manifest(|t| t + "binds = [\"Court\"]\n"),
        ),
        ("guidance without binds", manifest(kind("guidance"))),
        (
            "guidance binds nobody, or a stranger",
            manifest(|t| kind("guidance")(t) + "binds = []\n"),
        ),
        (
            "guidance binds a stranger",
            manifest(|t| kind("guidance")(t) + "binds = [\"Court\", \"Clerk\"]\n"),
        ),
        (
            "links package",
            manifest(|t| {
                "[package]\nname = \"links\"\nversion = \"0.1.0\"\nlanguage = 1\nin_force = \"0001-01-01\"\nknown_at = \"2026-10-19\"\nkind = \"amendment\"\n".to_owned()
                    + &t[t.find("\n[package.instrument]").unwrap_or(0)..]
            }),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn a_parsed_class_is_typed_and_base_is_omitted_from_hir() {
    let parse = |t: String| parse_manifest(&t).ok().map(|p| p.manifest);
    let standing = parse(layer("standing-order", "StandingOrder", "", "").manifest);
    let standing = standing.expect("valid");
    assert_eq!(standing.kind, Kind::StandingOrder);
    assert_eq!(
        standing.instrument.as_ref().map(|i| i.issuer.as_str()),
        Some("Demo Court")
    );
    let precedent = parse(
        layer(
            "precedent",
            "CaseOrder",
            "designation = \"1 F.4th 1\"\ncourt = \"ca9\"\n",
            "",
        )
        .manifest,
    )
    .expect("valid");
    // Binding scope is computed, never declared: fail closed.
    assert_eq!(
        precedent.kind,
        Kind::Precedent {
            scope: Scope::Persuasive
        }
    );
    let json = serde_json::to_value(&precedent).expect("json");
    assert_eq!(json["kind"]["class"], "precedent");
    assert_eq!(json["instrument"]["designation"], "1 F.4th 1");
    let guidance =
        parse(layer("guidance", "CaseOrder", "binds = [\"Court\"]\n", "").manifest).expect("valid");
    assert_eq!(
        guidance.kind,
        Kind::Guidance {
            binds: vec!["Court".to_owned()]
        }
    );
    let base = parse(pkg("b", "FederalRule", "2009-12-01", &[], &[]).manifest).expect("valid");
    let json = serde_json::to_value(&base).expect("json");
    assert!(json.get("kind").is_none() && json.get("instrument").is_none());
    assert!(base.kind.is_base());
}

#[test]
fn the_rows_are_the_matrix() {
    let table: Vec<String> = Kind::classes()
        .iter()
        .map(|k| {
            format!(
                "{:<20} {:<14} {}",
                k.label(),
                k.level().unwrap_or("-"),
                k.row()
                    .effects()
                    .map(Effect::name)
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        })
        .collect();
    insta::assert_debug_snapshot!(table);
    let what: Vec<String> = Effect::ALL
        .iter()
        .map(|e| {
            format!(
                "{}: cannot {}; permitted to {}",
                e.name(),
                e.what(),
                Kind::permitting(*e).join(", ")
            )
        })
        .collect();
    insta::assert_debug_snapshot!(what);
    // The names `Lit.toml` accepts are the classes', binding precedent
    // being a scope of `precedent`.
    let mut names: Vec<&str> = Kind::classes().iter().map(Kind::name).collect();
    names.dedup();
    assert_eq!(names, Kind::NAMES);
    // Rows are sets: inclusion, with and without, are exact.
    let row = Row::of(&[Effect::Declare, Effect::Rank]);
    assert!(row.has(Effect::Rank) && !row.without(Effect::Rank).has(Effect::Rank));
    assert_eq!(row.without(Effect::Rank).with(Effect::Rank), row);
    assert_eq!(Row::ALL.effects().count(), Effect::ALL.len());
    assert_eq!(Row::default().effects().count(), 0);
    // Every class except guidance sets any actor's prior its row allows.
    assert!(Kind::CaseOrder.may_set_prior("anyone"));
    assert!(!Kind::Overruling.may_set_prior("anyone"));
    let guidance = Kind::Guidance {
        binds: vec!["Court".to_owned()],
    };
    assert!(guidance.may_set_prior("Court") && !guidance.may_set_prior("Defendant"));
    let json = serde_json::to_string(&Effect::GrantCap).expect("json");
    assert_eq!(json, "\"grant-cap\"");
}

/// A filing limit prescribes (SPEC §3.7, §5.7): a local rule may state
/// one, persuasive precedent may not.
#[test]
fn a_filing_limit_prescribes() {
    let limit = "limit brief { words <= 13_000 }";
    let found = diagnostics(&[
        ("local-rule", layer("local-rule", "LocalRule", "", limit)),
        (
            "precedent",
            layer("precedent", "CaseOrder", "court = \"ca9\"\n", limit),
        ),
    ]);
    assert_eq!(found["local-rule"], Vec::<String>::new());
    assert!(
        found["precedent"].iter().any(|d| d.contains("L2010")),
        "{found:#?}"
    );
}
