//! Legal change as layered instruments (docs/litigate/CHANGE.md; SPEC
//! §5.7, §5.8): the forum-family class matrix `core` declares as data, a
//! precedent's binding scope computed from it, and worlds. Every illegal
//! class-to-operation pair the matrix adds to the class rows has a
//! negative test here (the rows' own are in `kinds.rs`): a class its
//! family lacks (L2011), a row a cell narrows (L2010), an emergency grant
//! beyond its window (L2011), persuasive precedent acting as binding
//! (L2010; L9009 in a world where it does not bind), an overruling of what
//! its court does not bind (L2010), and a counterfactual world or a
//! synthetic layer where only actual law is admitted (L1014).
mod legal;

use legal::{pkg, render, workspace, CORE};
use litigate_base::{FileId, LineIndex, Profile};
use litigate_sema::change::choice;
use litigate_sema::change::world::{check_world, parse_world, Modality, WorldCheck};
use litigate_sema::change::{class_index, key_range};
use litigate_sema::db::span::TextRange;
use litigate_sema::hir::{Effect, Kind, Scope};
use litigate_sema::pkg::load::LoadedPackage;
use litigate_sema::pkg::manifest::parse_manifest;
use std::fmt::Write as _;

const START: &str = "module core.start\nstart z\nstate z: \"Z\"\n";

/// The matrix of a two-family test library: `civil` (district courts in
/// circuits `ca1` and `ca2`) and `claims` (no local rules, no circuits).
const FAMILIES: &str = r#"
[family.civil]
title = "Civil"
forums = ["civil"]
precedent = ["scotus", "@circuit"]
circuits = ["ca1", "ca2"]

[family.civil.class.base]
cite = "Rules"

[family.civil.class.amendment]
cite = "28 U.S.C. § 2074"

[family.civil.class.local-rule]
cite = "28 U.S.C. § 2071"
example = "https://example.gov/local-rules"

[family.civil.class.standing-order]
cite = "FRAP 47(a)(1)"
row = ["declare"]

[family.civil.class.case-order]
cite = "FRCP 16(b)"

[family.civil.class.emergency-declaration]
cite = "FRCP 87(b)(1)(C)"
window_days = 90

[family.civil.class.guidance]
cite = "Committee Notes"

[family.civil.class.precedent]
cite = "Hutto v. Davis"

[family.civil.class.overruling]
cite = "Rodriguez de Quijas"

[family.claims]
title = "Claims"
forums = ["claims"]
precedent = ["scotus", "fed"]

[family.claims.class.base]
cite = "28 U.S.C. § 2503(b)"

[family.claims.class.general-order]
cite = "RCFC 83(b)"

[family.claims.class.precedent]
cite = "28 U.S.C. § 1295(a)(3)"
"#;

fn core() -> LoadedPackage {
    let mut p = pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[("authority.lit", CORE), ("start.lit", START)],
    );
    p.manifest.push_str(FAMILIES);
    p
}

/// `fed` (`FederalRule`, forum `civil`): a deadline delegated to a local
/// rule and to a case order, and a default.
fn fed() -> LoadedPackage {
    let text = "module fed.rules
import core.authority.{LocalRule, CaseOrder}
start s
event Served()
counting c6 { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
deadline answer-period: 21.days from Served() counted_by c6 effective [2009-12-01, ..) delegable { to LocalRule, op set } delegable { to CaseOrder, op set }
default open when true => true
state s: \"S\" { _ \"go\" -> t }
terminal t: \"T\"
";
    let mut p = pkg(
        "fed",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[("rules.lit", text)],
    );
    p.manifest = forum(&p.manifest, "civil");
    p
}

fn forum(manifest: &str, forum: &str) -> String {
    manifest.replacen("forum = \"demo\"", &format!("forum = \"{forum}\""), 1)
}

/// A layer `name` of class `kind` in `forum`, with `authority`, `extra`
/// instrument keys and `body` declarations, over `fed`.
fn layer(
    name: &str,
    kind: &str,
    forum_id: &str,
    authority: &str,
    extra: &str,
    body: &str,
) -> LoadedPackage {
    let module = name.replace('-', "_");
    let text = format!(
        "module {module}.rules
import core.authority.{{CaseOrder}}
import fed.rules.{{answer-period, open, c6, cal, Served}}
start l
state l: \"L\"
{body}
"
    );
    let mut p = pkg(
        name,
        authority,
        "2009-12-01",
        &["core", "fed"],
        &[("rules.lit", &text)],
    );
    p.manifest = forum(&p.manifest, forum_id).replacen(
        "title = \"T\"\n",
        &format!("title = \"T\"\nkind = \"{kind}\"\n"),
        1,
    );
    let _ = write!(
        p.manifest,
        "\n[sources.order]\npath = \"sources/order.txt\"\n\n[package.instrument]\nsource = \"order\"\nissuer = \"Demo\"\ndate = \"2020-01-01\"\n{extra}"
    );
    p
}

/// The diagnostics but lints of a workspace of `core`, `fed` and `layers`,
/// outside `core` and `fed`.
fn diagnostics(layers: &[LoadedPackage]) -> Vec<String> {
    let mut all = vec![core(), fed()];
    all.extend(layers.iter().cloned());
    let (db, ws) = workspace(&all);
    render(&db, ws)
        .into_iter()
        .filter(|d| !d.contains(" L9") && !d.starts_with("pkgs/core") && !d.starts_with("pkgs/fed"))
        .collect()
}

#[test]
fn the_matrix_is_data_that_only_narrows() {
    let bad = |edit: &str| -> Vec<String> {
        let text = core().manifest.replace(FAMILIES, edit);
        let index = LineIndex::new(&text);
        let file = FileId("Lit.toml".to_owned());
        parse_manifest(&text)
            .err()
            .unwrap_or_default()
            .iter()
            .map(|r| {
                let d = r.to_diagnostic(&file, &index);
                format!("{} {}", d.code.as_str(), d.message)
            })
            .collect()
    };
    let rows: Vec<(&str, Vec<String>)> = vec![
        ("valid", bad(FAMILIES)),
        (
            "every defect",
            bad(r#"
[family.Bad_Id]
title = "B"
forums = []
precedent = ["@circuit", "Not Kebab"]
circuits = ["CA 1"]

[family.Bad_Id.class.local-rule]
cite = " "
example = "http://insecure"
row = ["rank", "teleport"]
window_days = 3

[family.Bad_Id.class.hypothetical]
cite = "none"

[family.Bad_Id.class.emergency-declaration]
cite = "FRCP 87"
window_days = 0

[family.other]
title = "O"
forums = ["x", "x"]
precedent = ["@circuit"]

[family.other.class.base]
cite = "c"
"#),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
    // Only `core` declares the matrix.
    let mut stray = pkg("other", "FederalRule", "2009-12-01", &[], &[]);
    stray.manifest.push_str(FAMILIES);
    let found: Vec<String> = parse_manifest(&stray.manifest)
        .err()
        .unwrap_or_default()
        .iter()
        .map(|r| r.message.clone())
        .collect();
    assert_eq!(
        found,
        ["only `core` declares the forum-family matrix (`[family.*]`)"]
    );
    // The parsed matrix keeps every cell, and a cell's row is the class's
    // narrowed.
    let m = parse_manifest(&core().manifest).expect("valid").manifest;
    let civil = &m.families[0];
    assert_eq!(civil.classes.len(), 9);
    let standing = civil.cell(&Kind::StandingOrder).expect("cell");
    assert_eq!(
        standing
            .row(Kind::StandingOrder.row())
            .effects()
            .collect::<Vec<_>>(),
        [Effect::Declare]
    );
    let local = civil.cell(&Kind::LocalRule).expect("cell");
    assert_eq!(local.row(Kind::LocalRule.row()), Kind::LocalRule.row());
    assert!(civil.binds("ca1", Some("ca1")) && !civil.binds("ca1", Some("ca2")));
    assert!(civil.binds("ca2", None) && !civil.binds("ca3", None));
    assert!(civil.binds("scotus", Some("ca2")));
}

#[test]
fn a_class_its_family_lacks_or_narrows_does_not_type_check() {
    let rows: Vec<(&str, Vec<String>)> = vec![
        // COFC has no local-rule layer (28 U.S.C. § 2503(b)).
        (
            "local rule where the family has none",
            diagnostics(&[layer("lr", "local-rule", "claims", "LocalRule", "", "")]),
        ),
        (
            "a forum in no family",
            diagnostics(&[layer("lr", "local-rule", "nowhere", "LocalRule", "", "")]),
        ),
        // FRAP 47(a)(1): a circuit standing order gives no generally
        // applicable direction.
        (
            "a narrowed standing order prescribes",
            diagnostics(&[layer(
                "so",
                "standing-order",
                "civil",
                "StandingOrder",
                "",
                "deadline mine: 3.days from Served() counted_by c6",
            )]),
        ),
        (
            "a narrowed standing order declares",
            diagnostics(&[layer(
                "so",
                "standing-order",
                "civil",
                "StandingOrder",
                "",
                "event Mine()",
            )]),
        ),
        (
            "a local rule overrides what is delegated to it",
            diagnostics(&[layer(
                "lr",
                "local-rule",
                "civil",
                "LocalRule",
                "",
                "deadline mine: 30.days from Served() counted_by c6 override answer-period",
            )]),
        ),
        (
            "a base package in a forum with no family",
            diagnostics(&[{
                let mut p = pkg(
                    "loose",
                    "FederalRule",
                    "2009-12-01",
                    &["core"],
                    &[("rules.lit", "module loose.rules\nstart l\nstate l: \"L\"\n")],
                );
                p.manifest = forum(&p.manifest, "nowhere");
                p
            }]),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn an_emergency_grant_is_bounded_by_its_window() {
    let grant = |window: &str| {
        layer(
            "emergency",
            "emergency-declaration",
            "civil",
            "FederalRule",
            "",
            &format!("grant g on [answer-period] {window} cite \"FRCP 87\" {{ unlock }}"),
        )
    };
    let rows: Vec<(&str, Vec<String>)> = vec![
        // FRCP 87(b)(1)(C): no more than 90 days.
        (
            "90 days",
            diagnostics(&[grant("effective [2020-03-15, 2020-06-13)")]),
        ),
        (
            "91 days",
            diagnostics(&[grant("effective [2020-03-15, 2020-06-14)")]),
        ),
        (
            "open-ended",
            diagnostics(&[grant("effective [2020-03-15, ..)")]),
        ),
        (
            "an emergency declaration prescribes",
            diagnostics(&[layer(
                "emergency",
                "emergency-declaration",
                "civil",
                "FederalRule",
                "",
                "deadline mine: 3.days from Served() counted_by c6",
            )]),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
}

/// A precedent of `court`, designated when `designated`, with `body`.
fn precedent(name: &str, court: &str, designated: bool, body: &str) -> LoadedPackage {
    let extra = format!(
        "court = \"{court}\"\n{}",
        if designated {
            "designated = \"2020-02-01\"\n"
        } else {
            ""
        }
    );
    layer(name, "precedent", "civil", "CaseOrder", &extra, body)
}

const INTERPRETS: &str = "exception e rebuts open when true => false";

#[test]
fn a_precedents_scope_is_computed_from_its_court() {
    let rows: Vec<(&str, Vec<String>)> = vec![
        (
            "a designated Supreme Court precedent interprets",
            diagnostics(&[precedent("p", "scotus", true, INTERPRETS)]),
        ),
        (
            "a designated circuit precedent interprets",
            diagnostics(&[precedent("p", "ca1", true, INTERPRETS)]),
        ),
        // Camreta v. Greene: a district decision binds no one.
        (
            "a district decision interprets",
            diagnostics(&[precedent("p", "d-mass", true, INTERPRETS)]),
        ),
        // Fed. Cir. R. 32.1(d): a nonprecedential disposition is not binding.
        (
            "a nonprecedential decision interprets",
            diagnostics(&[precedent("p", "scotus", false, INTERPRETS)]),
        ),
        // A binding precedent's override is its waiver, which still needs
        // the target's delegation to its rung (L3004 without one).
        (
            "a binding precedent waives what is delegated to its rung",
            diagnostics(&[precedent(
                "p",
                "scotus",
                true,
                "deadline waived: 30.days from Served() counted_by c6 override answer-period",
            )]),
        ),
        (
            "a binding precedent waives what is not delegated to its rung",
            diagnostics(&[layer(
                "p",
                "precedent",
                "civil",
                "StandingOrder",
                "court = \"scotus\"\ndesignated = \"2020-02-01\"\n",
                "deadline waived: 30.days from Served() counted_by c6 override answer-period",
            )]),
        ),
        (
            "a binding precedent's waiver delegates further",
            diagnostics(&[precedent(
                "p",
                "scotus",
                true,
                "deadline waived: 30.days from Served() counted_by c6 override answer-period delegable { to CaseOrder, op set }",
            )]),
        ),
        (
            "a binding precedent prescribes",
            diagnostics(&[precedent(
                "p",
                "scotus",
                true,
                "deadline mine: 30.days from Served() counted_by c6",
            )]),
        ),
        (
            "a designation before the decision",
            diagnostics(&[layer(
                "p",
                "precedent",
                "civil",
                "CaseOrder",
                "court = \"scotus\"\ndesignated = \"2019-01-01\"\n",
                "",
            )]),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
    let all = [core(), fed(), precedent("p", "ca1", true, "")];
    let (db, ws) = workspace(&all);
    let p = ws.packages(&db)[2];
    let ix = class_index(&db, p).as_ref().expect("valid");
    assert_eq!(
        ix.kind,
        Kind::Precedent {
            scope: Scope::Binding
        }
    );
    assert_eq!(ix.family.as_ref().map(|f| f.0.as_str()), Some("civil"));
}

#[test]
fn a_decisions_instrument_names_its_court_and_what_it_overrules() {
    let parse = |kind: &str, extra: &str| -> Vec<String> {
        let p = layer("d", kind, "civil", "CaseOrder", extra, "");
        parse_manifest(&p.manifest)
            .err()
            .unwrap_or_default()
            .iter()
            .map(|r| r.message.clone())
            .collect()
    };
    let rows: Vec<(&str, Vec<String>)> = vec![
        ("precedent without a court", parse("precedent", "")),
        (
            "overruling without a court or target",
            parse("overruling", ""),
        ),
        (
            "overruling of nothing",
            parse("overruling", "court = \"scotus\"\noverrules = []\n"),
        ),
        (
            "a court on a case order",
            parse("case-order", "court = \"scotus\"\n"),
        ),
        (
            "a designation and an overruled package on a case order",
            parse(
                "case-order",
                "designated = \"2020-01-01\"\noverrules = [\"p\"]\n",
            ),
        ),
        (
            "a bad court id and designation date",
            parse(
                "precedent",
                "court = \"Supreme Court\"\ndesignated = \"2020-02-30\"\n",
            ),
        ),
        (
            "a synthetic layer",
            parse("case-order", "authoritative = false\n"),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
    let m = parse_manifest(
        &layer(
            "d",
            "overruling",
            "civil",
            "CaseOrder",
            "court = \"scotus\"\noverrules = [\"p\"]\nauthoritative = false\n",
            "",
        )
        .manifest,
    )
    .expect("valid")
    .manifest;
    let i = m.instrument.expect("instrument");
    assert_eq!(
        (i.overrules, i.authoritative),
        (vec!["p".to_owned()], false)
    );
}

/// A workspace of `core`, `fed` and `layers`, and its world check of
/// `text` under `profile`.
fn world(layers: &[LoadedPackage], text: &str, profile: Profile) -> WorldCheck {
    let mut all = vec![core(), fed()];
    all.extend(layers.iter().cloned());
    let (db, ws) = workspace(&all);
    check_world(&db, ws, "w.world", text, profile)
}

/// `line:col CODE message` of every diagnostic of `w`, and its layers as
/// `name class in_force [overruled_by]`.
fn shown(w: &WorldCheck) -> Vec<String> {
    w.diagnostics
        .iter()
        .map(|d| {
            format!(
                "{}:{}:{} {} {}",
                d.span.file.0,
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message
            )
        })
        .chain(w.layers.iter().map(|l| {
            format!(
                "layer {} {} {}{}",
                l.name,
                l.class,
                if l.in_force {
                    "in force"
                } else {
                    "not in force"
                },
                l.overruled_by
                    .as_ref()
                    .map(|o| format!(" (overruled by {} from {})", o.by, o.from))
                    .unwrap_or_default()
            )
        }))
        .collect()
}

fn located(layers: &str, location: &str) -> String {
    format!("layers = [{layers}]\n\n[location]\n{location}\n")
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn a_world_file_is_checked_against_its_family() {
    let lr = layer("lr", "local-rule", "civil", "LocalRule", "", "");
    let rows: Vec<(&str, Vec<String>)> = vec![
        ("not toml", shown(&world(&[], "layers = [", Profile::Dev))),
        (
            "unknown key",
            shown(&world(
                &[],
                "layers = [\"fed\"]\nplace = 1\n[location]\nfamily = \"civil\"\n",
                Profile::Dev,
            )),
        ),
        (
            "unknown modality",
            shown(&world(
                &[],
                &format!(
                    "modality = \"hypothetical\"\n{}",
                    located("\"fed\"", "family = \"civil\"\ncircuit = \"ca1\"")
                ),
                Profile::Dev,
            )),
        ),
        (
            "no layers",
            shown(&world(
                &[],
                &located("", "family = \"civil\""),
                Profile::Dev,
            )),
        ),
        (
            "an unknown layer and family",
            shown(&world(
                &[],
                &located("\"fed\", \"ghost\"", "family = \"criminal\""),
                Profile::Dev,
            )),
        ),
        (
            "a circuit family without a circuit",
            shown(&world(
                &[],
                &located("\"fed\"", "family = \"civil\""),
                Profile::Dev,
            )),
        ),
        (
            "a circuit outside the family",
            shown(&world(
                &[],
                &located("\"fed\"", "family = \"civil\"\ncircuit = \"ca9\""),
                Profile::Dev,
            )),
        ),
        (
            "a circuit in a family without circuits",
            shown(&world(
                &[],
                &located("\"fed\"", "family = \"claims\"\ncircuit = \"ca1\""),
                Profile::Dev,
            )),
        ),
        (
            "a family without circuits, and none named",
            shown(&world(
                &[],
                &located("\"fed\"", "family = \"claims\""),
                Profile::Dev,
            )),
        ),
        (
            "an actual world with a local rule",
            shown(&world(
                &[lr],
                &located(
                    "\"lr\", \"fed\"",
                    "family = \"civil\"\ncourt = \"d-mass\"\ncircuit = \"ca1\"\njudge = \"J\"",
                ),
                Profile::Strict,
            )),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn a_counterfactual_world_is_never_strict_and_synthetic_law_never_actual() {
    let what_if = layer(
        "what-if",
        "emergency-declaration",
        "civil",
        "FederalRule",
        "authoritative = false\n",
        "grant g on [answer-period] effective [2020-03-15, 2020-06-13) cite \"FRCP 87\" { unlock }",
    );
    let at = "family = \"civil\"\ncircuit = \"ca2\"";
    let counterfactual = format!(
        "modality = \"counterfactual\"\n{}",
        located("\"what-if\"", at)
    );
    let rows: Vec<(&str, Vec<String>)> = vec![
        (
            "a synthetic layer in an actual world",
            shown(&world(
                std::slice::from_ref(&what_if),
                &located("\"what-if\"", at),
                Profile::Dev,
            )),
        ),
        (
            "a counterfactual world under strict",
            shown(&world(
                std::slice::from_ref(&what_if),
                &counterfactual,
                Profile::Strict,
            )),
        ),
        (
            "a counterfactual world under dev",
            shown(&world(
                std::slice::from_ref(&what_if),
                &counterfactual,
                Profile::Dev,
            )),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
    let (w, _) = parse_world(&counterfactual).expect("parses");
    assert_eq!(w.modality, Modality::Counterfactual);
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn precedent_binds_per_circuit_and_an_overruling_withdraws_it() {
    let ca1 = precedent("ca1-rule", "ca1", true, INTERPRETS);
    let quiet = precedent("ca1-dicta", "ca1", true, "event Noted()");
    let at = |circuit: &str| format!("family = \"civil\"\ncircuit = \"{circuit}\"");
    let overruling = |name: &str, court: &str, target: &str| {
        layer(
            name,
            "overruling",
            "civil",
            "CaseOrder",
            &format!("court = \"{court}\"\noverrules = [\"{target}\", \"absent\"]\n"),
            "",
        )
    };
    let base_case = layer("order", "case-order", "civil", "CaseOrder", "", "");
    let packs = [
        ca1.clone(),
        quiet.clone(),
        overruling("en-banc", "ca1", "ca1-rule"),
        overruling("scotus-rev", "scotus", "ca1-rule"),
        overruling("sister", "ca2", "ca1-rule"),
        overruling("of-an-order", "scotus", "order"),
        base_case,
    ];
    let rows: Vec<(&str, Vec<String>)> = vec![
        (
            "in its own circuit",
            shown(&world(
                &packs,
                &located("\"ca1-rule\", \"ca1-dicta\"", &at("ca1")),
                Profile::Dev,
            )),
        ),
        // The lg-fq2.3 circuit splits differ by circuit: elsewhere it is
        // persuasive, and its interpretation is not in force.
        (
            "in a sister circuit",
            shown(&world(
                &packs,
                &located("\"ca1-rule\", \"ca1-dicta\"", &at("ca2")),
                Profile::Dev,
            )),
        ),
        (
            "overruled en banc",
            shown(&world(
                &packs,
                &located("\"ca1-rule\", \"en-banc\"", &at("ca1")),
                Profile::Dev,
            )),
        ),
        (
            "overruled by the Supreme Court",
            shown(&world(
                &packs,
                &located("\"ca1-rule\", \"scotus-rev\"", &at("ca1")),
                Profile::Dev,
            )),
        ),
        (
            "a sister circuit cannot overrule it",
            shown(&world(
                &packs,
                &located("\"ca1-rule\", \"sister\"", &at("ca1")),
                Profile::Dev,
            )),
        ),
        (
            "never designated, in its own circuit",
            shown(&world(
                &[precedent("undesignated", "ca1", false, INTERPRETS)],
                &located("\"undesignated\"", &at("ca1")),
                Profile::Dev,
            )),
        ),
        (
            "at a location in no family",
            shown(&world(
                &packs,
                &located("\"ca1-rule\"", "family = \"nowhere\""),
                Profile::Dev,
            )),
        ),
        (
            "an overruling of an order",
            shown(&world(
                &packs,
                &located("\"order\", \"of-an-order\"", &at("ca1")),
                Profile::Dev,
            )),
        ),
    ];
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn layers_merge_as_a_set() {
    let lr = layer(
        "lr",
        "local-rule",
        "civil",
        "LocalRule",
        "",
        "deadline local-answer: 30.days from Served() counted_by c6 override answer-period",
    );
    let order = layer("order", "case-order", "civil", "CaseOrder", "", "");
    let at = "family = \"civil\"\ncircuit = \"ca1\"";
    let packs = [lr, order];
    let one = world(
        &packs,
        &located("\"lr\", \"order\", \"fed\"", at),
        Profile::Dev,
    );
    // Order and repetition in the file mean nothing: the same world, the
    // same check.
    for layers in [
        "\"fed\", \"order\", \"lr\"",
        "\"order\", \"lr\", \"fed\", \"lr\", \"order\"",
    ] {
        let other = world(&packs, &located(layers, at), Profile::Dev);
        assert_eq!(other.world, one.world);
        assert_eq!(other.layers, one.layers);
        assert_eq!(shown(&other), shown(&one));
    }
    assert_eq!(one.layers.len(), 3);
    assert!(one.diagnostics.is_empty(), "{:?}", shown(&one));
}

#[test]
fn a_conflict_only_the_merged_layers_have_is_reported() {
    // Two local rules of the same rung each override the delegated period,
    // differently: alone each is fine; together nothing orders them.
    let local = |name: &str, days: u32| {
        layer(
            name,
            "local-rule",
            "civil",
            "LocalRule",
            "",
            &format!(
                "deadline {name}-answer: {days}.days from Served() counted_by c6 override answer-period"
            ),
        )
    };
    let packs = [local("east", 30), local("west", 45)];
    let at = "family = \"civil\"\ncircuit = \"ca1\"";
    let alone: Vec<String> = diagnostics(&packs)
        .into_iter()
        .filter(|d| d.contains(" L3"))
        .collect();
    assert!(alone.is_empty(), "{alone:?}");
    let merged = world(&packs, &located("\"east\", \"west\"", at), Profile::Dev);
    let codes: Vec<&str> = merged.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"L3003"), "{:?}", shown(&merged));
}

#[test]
fn a_key_is_found_on_its_own_line() {
    let text = "[package]\nkinder = 1\n  kind = \"x\"\n";
    let r = key_range(text, "kind");
    assert_eq!(&text[r.start as usize..r.end as usize], "kind = \"x\"");
    assert_eq!(key_range(text, "absent"), TextRange::default());
}

#[test]
fn a_world_reads_the_newest_valid_version_of_each_layer() {
    let mut old = layer("lr", "local-rule", "civil", "LocalRule", "", "");
    old.root = "pkgs/lr-old".to_owned();
    let mut new = layer("lr", "local-rule", "civil", "LocalRule", "", "");
    new.root = "pkgs/lr-new".to_owned();
    new.manifest = new
        .manifest
        .replacen("version = \"1.0.0\"", "version = \"1.1.0\"", 1);
    let mut broken = layer("broken", "local-rule", "civil", "LocalRule", "", "");
    broken.manifest = broken
        .manifest
        .replace("[package]", "[package]\nnot_a_key = 1");
    let at = located("\"lr\"", "family = \"civil\"\ncircuit = \"ca1\"");
    for order in [[old.clone(), new.clone()], [new.clone(), old.clone()]] {
        let mut packs = order.to_vec();
        packs.push(broken.clone());
        let w = world(&packs, &at, Profile::Dev);
        assert_eq!(w.layers[0].version, "1.1.0");
    }
    // An instrument read back without `authoritative` is authoritative.
    let i: litigate_sema::hir::Instrument =
        serde_json::from_str(r#"{"source": "s", "issuer": "i", "date": "2020-01-01"}"#)
            .expect("instrument");
    assert!(i.authoritative && i.overrules.is_empty());
    assert_eq!(
        serde_json::to_value(&i).expect("json"),
        serde_json::json!({"source": "s", "issuer": "i", "date": "2020-01-01"})
    );
}

/// `w.world` with the world `located` gives, then `choices`.
fn chosen(layers: &str, location: &str, choices: &str) -> String {
    format!("{}{choices}", located(layers, location))
}

const CA1: &str = "family = \"civil\"\ncircuit = \"ca1\"";

#[test]
fn choice_points_span_one_world_per_combination() {
    let lr = layer("lr", "local-rule", "civil", "LocalRule", "", "");
    let co = layer("co", "case-order", "civil", "CaseOrder", "", "");
    let co2 = layer("co2", "case-order", "civil", "CaseOrder", "", "");
    let text = chosen(
        "\"fed\"",
        CA1,
        "\n[choice.local]\nwithout = []\nwith = [\"lr\"]\n\n[choice.order]\nnone = []\nentered = [\"co\", \"co2\"]\nalone = [\"co\"]\n",
    );
    // `co` is in two options of one choice: legal, one choice owns it.
    let (w, at) = parse_world(&text).unwrap();
    let combos: Vec<String> = choice::combinations(&w)
        .iter()
        .map(|c| format!("{:?} {:?}", c.choices, c.layers))
        .collect();
    let mut all = vec![core(), fed(), lr, co, co2];
    let (db, ws) = workspace(&all);
    let checked = check_world(&db, ws, "w.world", &text, Profile::Dev);
    // The world's own layers are checked; its choices' are the worlds'.
    assert_eq!(shown(&checked), ["layer fed base in force"]);
    let alternates = choice::enumerate(&db, ws, "w.world", &text, &w, &at, Profile::Dev);
    let ids: Vec<(String, Vec<String>)> = alternates
        .iter()
        .map(|a| (a.id.clone(), shown(&a.check)))
        .collect();
    insta::assert_debug_snapshot!((combos, ids));
    // The world reports its choices; a world without any reports none.
    let json = serde_json::to_value(checked.world.as_ref().unwrap()).unwrap();
    assert_eq!(json["choices"]["local"]["with"], serde_json::json!(["lr"]));
    all.truncate(2);
    let plain = world(&all, &located("\"fed\"", CA1), Profile::Dev);
    let json = serde_json::to_value(plain.world.as_ref().unwrap()).unwrap();
    assert!(json.get("choices").is_none(), "{json}");
    assert_eq!(choice::combinations(plain.world.as_ref().unwrap()).len(), 1);
}

#[test]
fn malformed_choice_points_are_l1013() {
    let many = (0..13).fold(String::new(), |mut s, i| {
        let _ = write!(s, "\n[choice.c{i}]\nwithout = []\nwith = [\"l{i}\"]\n");
        s
    });
    let rows: Vec<(&str, Vec<String>)> = [
        ("one option", "\n[choice.c]\nwith = [\"lr\"]\n"),
        (
            "an option naming the world's own layer",
            "\n[choice.c]\nwithout = []\nwith = [\"fed\"]\n",
        ),
        (
            "a layer in two choices",
            "\n[choice.a]\nwithout = []\nwith = [\"lr\"]\n\n[choice.b]\nwithout = []\nwith = [\"lr\"]\n",
        ),
        (
            "two options with the same layers",
            "\n[choice.c]\nfirst = [\"lr\"]\nagain = [\"lr\"]\n",
        ),
        ("more worlds than the limit", many.as_str()),
        (
            "an option layer that is not loaded",
            "\n[choice.c]\nwithout = []\nwith = [\"ghost\"]\n",
        ),
    ]
    .into_iter()
    .map(|(name, choices)| {
        let text = chosen("\"fed\"", CA1, choices);
        (name, shown(&world(&[], &text, Profile::Dev)))
    })
    .chain([(
        "a choice that is not a table",
        shown(&world(
            &[],
            &format!("choice = 1\n{}", located("\"fed\"", CA1)),
            Profile::Dev,
        )),
    )])
    .collect();
    insta::assert_debug_snapshot!(rows);
}

#[test]
fn a_combination_with_an_error_is_reported_in_its_own_world() {
    let what_if = layer(
        "what-if",
        "case-order",
        "civil",
        "CaseOrder",
        "authoritative = false\n",
        "",
    );
    let text = chosen(
        "\"fed\"",
        CA1,
        "\n[choice.c]\nwithout = []\nwith = [\"what-if\"]\n",
    );
    let (db, ws) = workspace(&[core(), fed(), what_if]);
    let (w, at) = parse_world(&text).unwrap();
    let errors: Vec<(String, Vec<String>)> =
        choice::enumerate(&db, ws, "w.world", &text, &w, &at, Profile::Dev)
            .iter()
            .map(|a| (a.id.clone(), shown(&a.check)))
            .collect();
    insta::assert_debug_snapshot!(errors);
}

#[test]
fn an_overruled_precedent_is_in_force_until_the_overruling() {
    let ca1 = precedent("ca1-rule", "ca1", true, INTERPRETS);
    let en_banc = layer(
        "en-banc",
        "overruling",
        "civil",
        "CaseOrder",
        "court = \"ca1\"\noverrules = [\"ca1-rule\"]\n",
        "",
    );
    let at = |circuit: &str| format!("family = \"civil\"\ncircuit = \"{circuit}\"");
    let packs = [ca1, en_banc];
    let overruled = world(
        &packs,
        &located("\"ca1-rule\", \"en-banc\"", &at("ca1")),
        Profile::Dev,
    );
    let rule = overruled
        .layers
        .iter()
        .find(|l| l.name == "ca1-rule")
        .unwrap();
    // Overruled from the overruling's date (2020-01-01), for every case
    // still open then; a matter judged earlier, or at a `known_at`
    // before it, still has it.
    let got: Vec<bool> = [
        ("2019-12-31", None),
        ("2020-01-01", None),
        ("2021-01-01", Some("2019-06-01")),
        ("2021-01-01", Some("2020-06-01")),
    ]
    .iter()
    .map(|(as_of, known_at)| rule.in_force_at(as_of, *known_at))
    .collect();
    assert_eq!(got, [true, false, true, false]);
    // A precedent that does not bind at the location is never in force.
    let sister = world(&packs, &located("\"ca1-rule\"", &at("ca2")), Profile::Dev);
    assert!(!sister.layers[0].in_force_at("2019-12-31", None));
    // Any other layer is in force at every date.
    assert!(overruled
        .layers
        .iter()
        .find(|l| l.name == "en-banc")
        .unwrap()
        .in_force_at("2000-01-01", None));
}
