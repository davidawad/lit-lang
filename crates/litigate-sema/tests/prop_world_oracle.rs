//! An independent world-enumeration oracle (design review 2026-10-23
//! §1.2, §4.7; docs/litigate/CHANGE.md §5).
//!
//! A fixed change set of precedents, overrulings, a synthetic what-if and
//! two rival local rules is enumerated into every world `(Λ, ι, μ)`: each
//! subset of the layers at each circuit, in each modality, under each
//! profile. The oracle reads the instruments as plain facts (court,
//! designation, date, targets) and states the court hierarchy itself; it
//! never sees the manifest, the family matrix or the compiler's world
//! check, and every world's outcome (each layer's class, whether it is in
//! force and what withdraws it, and each finding) must equal
//! [`check_world`]'s. The review suggests clingo for this oracle; it is a
//! Rust generate-and-test so the gate needs no solver.
mod legal;

use legal::{pkg, workspace, CORE};
use litigate_base::Profile;
use litigate_sema::change::world::{check_world, WorldCheck};
use litigate_sema::pkg::load::LoadedPackage;
use litigate_sema::{LitDatabase, Workspace};
use std::collections::BTreeSet;
use std::fmt::Write as _;

const START: &str = "module core.start\nstart z\nstate z: \"Z\"\n";

/// One forum family: district courts in circuits `ca1` and `ca2`, bound
/// by the Supreme Court and their own circuit.
const FAMILY: &str = r#"
[family.civil]
title = "Civil"
precedent = ["scotus", "@circuit"]
circuits = ["ca1", "ca2"]

[family.civil.class.base]
cite = "Rules"

[family.civil.class.local-rule]
cite = "28 U.S.C. § 2071"

[family.civil.class.emergency-declaration]
cite = "FRCP 87(b)(1)(C)"
window_days = 90

[family.civil.class.precedent]
cite = "Hutto v. Davis"

[family.civil.class.overruling]
cite = "Rodriguez de Quijas"
"#;

/// What a layer is, as the oracle reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    LocalRule,
    Emergency,
    Precedent,
    Overruling,
}

/// One instrument of the change set.
#[derive(Clone, Copy, Debug)]
struct Instrument {
    name: &'static str,
    class: Class,
    court: &'static str,
    /// A precedent designated precedential.
    designated: bool,
    /// A precedent whose declarations only binding precedent may have
    /// (an interpretation).
    interprets: bool,
    /// False for a synthetic what-if.
    authoritative: bool,
    date: &'static str,
    overrules: &'static [&'static str],
}

const fn instrument(name: &'static str, class: Class) -> Instrument {
    Instrument {
        name,
        class,
        court: "",
        designated: false,
        interprets: false,
        authoritative: true,
        date: "2010-01-01",
        overrules: &[],
    }
}

/// The change set: a ca1 precedent, an undesignated one, a Supreme Court
/// one, an en banc overruling (2015) and a Supreme Court overruling
/// (2018) of the ca1 precedent, a sister circuit's overruling of it and
/// of a local rule, a synthetic emergency declaration, and two local
/// rules overriding the same period.
const SET: [Instrument; 10] = [
    Instrument {
        court: "ca1",
        designated: true,
        interprets: true,
        ..instrument("ca1-rule", Class::Precedent)
    },
    Instrument {
        court: "ca1",
        interprets: true,
        ..instrument("ca1-memo", Class::Precedent)
    },
    Instrument {
        court: "scotus",
        designated: true,
        interprets: true,
        date: "2011-01-01",
        ..instrument("high-rule", Class::Precedent)
    },
    Instrument {
        court: "ca1",
        date: "2015-03-01",
        overrules: &["ca1-rule"],
        ..instrument("en-banc", Class::Overruling)
    },
    Instrument {
        court: "scotus",
        date: "2018-06-01",
        overrules: &["ca1-rule", "high-rule", "absent"],
        ..instrument("scotus-rev", Class::Overruling)
    },
    Instrument {
        court: "ca2",
        date: "2016-01-01",
        overrules: &["ca1-rule", "east"],
        ..instrument("sister", Class::Overruling)
    },
    Instrument {
        authoritative: false,
        date: "2020-03-15",
        ..instrument("what-if", Class::Emergency)
    },
    instrument("east", Class::LocalRule),
    instrument("west", Class::LocalRule),
    // A designated circuit precedent with nothing only binding precedent
    // may do: in force wherever it is.
    Instrument {
        court: "ca1",
        designated: true,
        ..instrument("ca1-dicta", Class::Precedent)
    },
];

const CIRCUITS: [&str; 2] = ["ca1", "ca2"];

/// A world's location and modality, and the profile it is checked under.
#[derive(Clone, Copy, Debug)]
struct Coordinates {
    circuit: &'static str,
    counterfactual: bool,
    strict: bool,
}

/// One layer's outcome: `name class in force|not in force [(overruled by
/// X from D)]`, or a finding `CODE subject`.
type Outcome = Vec<String>;

/// The oracle's court hierarchy: the Supreme Court binds every court; a
/// circuit binds itself.
fn binds(court: &str, at: &str) -> bool {
    court == "scotus" || court == at
}

fn find(name: &str) -> Option<&'static Instrument> {
    SET.iter().find(|i| i.name == name)
}

/// The oracle's outcome of world `members` (indices into [`SET`]) at `c`.
fn reference(members: &BTreeSet<usize>, c: Coordinates) -> Outcome {
    let mut findings: BTreeSet<String> = BTreeSet::new();
    if members.is_empty() {
        findings.insert("L1013 layers".to_owned());
        return findings.into_iter().collect();
    }
    let present = |name: &str| members.iter().any(|&m| SET[m].name == name);
    if c.counterfactual && c.strict {
        findings.insert("L1014 modality".to_owned());
    }
    if present("east") && present("west") {
        findings.insert("L3003 east+west".to_owned());
    }
    // The valid overrulings of each precedent; the earliest withdraws it.
    let mut withdrawn: Vec<(&str, &str, &str)> = Vec::new();
    for &m in members {
        let o = &SET[m];
        for target in o.overrules {
            let Some(t) = find(target).filter(|t| present(t.name)) else {
                continue;
            };
            if t.class != Class::Precedent || !(o.court == t.court || binds(o.court, t.court)) {
                findings.insert(format!("L2010 {}", o.name));
                continue;
            }
            withdrawn.push((t.name, o.date, o.name));
        }
    }
    withdrawn.sort_unstable();
    let mut layers = Vec::new();
    let mut interpreting = 0;
    for &m in members {
        let l = &SET[m];
        if !l.authoritative && !c.counterfactual {
            findings.insert(format!("L1014 {}", l.name));
        }
        let class = match l.class {
            Class::LocalRule => "local-rule",
            Class::Emergency => "emergency-declaration",
            Class::Overruling => "overruling",
            Class::Precedent if l.designated && binds(l.court, c.circuit) => "binding precedent",
            Class::Precedent => "persuasive precedent",
        };
        let fits = !(l.interprets && class == "persuasive precedent");
        if l.interprets && fits {
            interpreting += 1;
        }
        if !fits {
            findings.insert(format!("L9009 {}", l.name));
        }
        let by = withdrawn.iter().find(|(t, ..)| *t == l.name);
        layers.push(format!(
            "layer {} {class} {}{}",
            l.name,
            if fits && by.is_none() {
                "in force"
            } else {
                "not in force"
            },
            by.map(|(_, from, o)| format!(" (overruled by {o} from {from})"))
                .unwrap_or_default()
        ));
    }
    // Two interpretations in force rebut the same default, unordered.
    if interpreting > 1 {
        findings.insert("L3005 exceptions".to_owned());
    }
    layers.sort();
    findings.into_iter().chain(layers).collect()
}

fn core() -> LoadedPackage {
    let mut p = pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[("authority.lit", CORE), ("start.lit", START)],
    );
    p.manifest.push_str(FAMILY);
    p
}

fn in_forum(manifest: &str) -> String {
    manifest.replacen(
        "forum = \"demo\"",
        "forum = \"civil\"\nfamily = \"civil\"",
        1,
    )
}

/// The base rules: a period delegated to local rules, and a default a
/// precedent may interpret.
fn fed() -> LoadedPackage {
    let text = "module fed.rules
import core.authority.{LocalRule}
start s
event Served()
counting c6 { holidays cal }
calendar cal { new-year: fixed(1, 1) effective [1870-06-28, ..) }
deadline answer-period: 21.days from Served() counted_by c6 effective [2009-12-01, ..) delegable { to LocalRule, op set }
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
    p.manifest = in_forum(&p.manifest);
    p
}

/// `i` as a package over `fed`.
fn package(i: &Instrument) -> LoadedPackage {
    let (kind, authority, body) = match i.class {
        Class::LocalRule => (
            "local-rule",
            "LocalRule",
            format!(
                "deadline {}-answer: {}.days from Served() counted_by c6 override answer-period",
                i.name,
                if i.name == "east" { 30 } else { 45 }
            ),
        ),
        Class::Emergency => (
            "emergency-declaration",
            "FederalRule",
            "grant g on [answer-period] effective [2020-03-15, 2020-06-13) cite \"FRCP 87\" { unlock }"
                .to_owned(),
        ),
        Class::Precedent if i.interprets => (
            "precedent",
            "CaseOrder",
            "exception e rebuts open when true => false".to_owned(),
        ),
        Class::Precedent | Class::Overruling => (
            if i.class == Class::Precedent {
                "precedent"
            } else {
                "overruling"
            },
            "CaseOrder",
            String::new(),
        ),
    };
    let module = i.name.replace('-', "_");
    let text = format!(
        "module {module}.rules\nimport fed.rules.{{answer-period, open, c6, cal, Served}}\nstart l\nstate l: \"L\"\n{body}\n"
    );
    let mut p = pkg(
        i.name,
        authority,
        "2009-12-01",
        &["core", "fed"],
        &[("rules.lit", &text)],
    );
    p.manifest = in_forum(&p.manifest).replacen(
        "title = \"T\"\n",
        &format!("title = \"T\"\nkind = \"{kind}\"\n"),
        1,
    );
    let _ = write!(
        p.manifest,
        "\n[sources.order]\npath = \"sources/order.txt\"\n\n[package.instrument]\nsource = \"order\"\nissuer = \"Demo\"\ndate = \"{}\"\n",
        i.date
    );
    if !i.court.is_empty() {
        let _ = writeln!(p.manifest, "court = \"{}\"", i.court);
    }
    if i.designated {
        let _ = writeln!(p.manifest, "designated = \"{}\"", i.date);
    }
    if !i.overrules.is_empty() {
        let targets: Vec<String> = i.overrules.iter().map(|t| format!("\"{t}\"")).collect();
        let _ = writeln!(p.manifest, "overrules = [{}]", targets.join(", "));
    }
    if !i.authoritative {
        p.manifest.push_str("authoritative = false\n");
    }
    p
}

/// The world file of `members` at `c`: one layer per line, so a finding's
/// line names what it is about.
fn world_text(members: &BTreeSet<usize>, c: Coordinates) -> String {
    let mut text = format!(
        "modality = \"{}\"\nlayers = [\n",
        if c.counterfactual {
            "counterfactual"
        } else {
            "actual"
        }
    );
    for &m in members.iter().rev() {
        let _ = writeln!(text, "  \"{}\",", SET[m].name);
    }
    let _ = write!(
        text,
        "]\n\n[location]\nfamily = \"civil\"\ncircuit = \"{}\"\n",
        c.circuit
    );
    text
}

/// What a diagnostic of `w` is about: the world-file line it is on, the
/// package whose manifest it is in, or the rival local rules.
fn subject(w: &WorldCheck, text: &str, d: &litigate_base::Diagnostic) -> String {
    match d.code.as_str() {
        "L3003" => return "east+west".to_owned(),
        "L3005" => return "exceptions".to_owned(),
        _ => {}
    }
    if d.span.file.0 == w.path {
        let line = text
            .lines()
            .nth(usize::try_from(d.span.line).unwrap_or(1) - 1)
            .unwrap_or_default();
        return line
            .split(['"', ' ', '='])
            .find(|s| !s.is_empty())
            .unwrap_or_default()
            .to_owned();
    }
    d.span
        .file
        .0
        .trim_start_matches("pkgs/")
        .split('/')
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// The compiler's outcome of world `members` at `c`.
fn compiled(db: &LitDatabase, ws: Workspace, members: &BTreeSet<usize>, c: Coordinates) -> Outcome {
    let text = world_text(members, c);
    let profile = if c.strict {
        Profile::Strict
    } else {
        Profile::Dev
    };
    let w = check_world(db, ws, "w.world", &text, profile);
    let findings: BTreeSet<String> = w
        .diagnostics
        .iter()
        .map(|d| format!("{} {}", d.code.as_str(), subject(&w, &text, d)))
        .collect();
    let mut layers: Vec<String> = w
        .layers
        .iter()
        .map(|l| {
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
        })
        .collect();
    layers.sort();
    findings.into_iter().chain(layers).collect()
}

/// The workspace of the whole change set.
fn change_set() -> (LitDatabase, Workspace) {
    let mut all = vec![core(), fed()];
    all.extend(SET.iter().map(package));
    workspace(&all)
}

fn members(names: &[&str]) -> BTreeSet<usize> {
    names
        .iter()
        .filter_map(|n| SET.iter().position(|i| i.name == *n))
        .collect()
}

const AT_CA1: Coordinates = Coordinates {
    circuit: "ca1",
    counterfactual: false,
    strict: false,
};

/// Every world of the change set: 2^10 layer sets, two circuits, two
/// modalities, two profiles.
#[test]
fn every_world_of_the_change_set_matches_the_oracle() {
    let (db, ws) = change_set();
    let mut disagreements = Vec::new();
    let mut outcomes: BTreeSet<String> = BTreeSet::new();
    for mask in 0_u32..(1 << SET.len()) {
        let members: BTreeSet<usize> = (0..SET.len()).filter(|i| mask & (1 << i) != 0).collect();
        for circuit in CIRCUITS {
            for (counterfactual, strict) in
                [(false, false), (false, true), (true, false), (true, true)]
            {
                let c = Coordinates {
                    circuit,
                    counterfactual,
                    strict,
                };
                let want = reference(&members, c);
                let got = compiled(&db, ws, &members, c);
                if got != want && disagreements.len() < 5 {
                    disagreements.push(format!(
                        "{}\nwant {want:#?}\ngot {got:#?}",
                        world_text(&members, c)
                    ));
                }
                outcomes.extend(got);
            }
        }
    }
    assert!(disagreements.is_empty(), "{}", disagreements.join("\n\n"));
    // The enumeration reaches every outcome the oracle distinguishes.
    for outcome in [
        "L1013 layers",
        "L1014 modality",
        "L1014 what-if",
        "L2010 sister",
        "L3003 east+west",
        "L3005 exceptions",
        "L9009 ca1-memo",
        "L9009 ca1-rule",
        "layer ca1-dicta persuasive precedent in force",
        "layer ca1-rule binding precedent not in force (overruled by en-banc from 2015-03-01)",
        "layer ca1-rule binding precedent not in force (overruled by scotus-rev from 2018-06-01)",
        "layer ca1-rule persuasive precedent not in force (overruled by en-banc from 2015-03-01)",
        "layer high-rule binding precedent not in force (overruled by scotus-rev from 2018-06-01)",
        "layer high-rule binding precedent in force",
    ] {
        assert!(outcomes.contains(outcome), "{outcome} never reached");
    }
}

/// Hand-picked worlds, with the oracle's outcome written out: two
/// precedents interpreting one default, in the first's own circuit and in
/// a sister circuit, an undesignated one, an invalid
/// overruling of a precedent and of a local rule, and the rival local
/// rules in a counterfactual world under strict.
#[test]
fn differential_cases_match_the_oracle() {
    let (db, ws) = change_set();
    let ca2 = Coordinates {
        circuit: "ca2",
        ..AT_CA1
    };
    let strict_what_if = Coordinates {
        counterfactual: true,
        strict: true,
        ..AT_CA1
    };
    let cases: [(&[&str], Coordinates, &[&str]); 5] = [
        (
            &["ca1-rule", "high-rule"],
            AT_CA1,
            &[
                "L3005 exceptions",
                "layer ca1-rule binding precedent in force",
                "layer high-rule binding precedent in force",
            ],
        ),
        (
            &["ca1-rule", "high-rule"],
            ca2,
            &[
                "L9009 ca1-rule",
                "layer ca1-rule persuasive precedent not in force",
                "layer high-rule binding precedent in force",
            ],
        ),
        (
            &["ca1-memo"],
            AT_CA1,
            &[
                "L9009 ca1-memo",
                "layer ca1-memo persuasive precedent not in force",
            ],
        ),
        (
            &["ca1-rule", "east", "sister"],
            AT_CA1,
            &[
                "L2010 sister",
                "layer ca1-rule binding precedent in force",
                "layer east local-rule in force",
                "layer sister overruling in force",
            ],
        ),
        (
            &["east", "west", "what-if"],
            strict_what_if,
            &[
                "L1014 modality",
                "L3003 east+west",
                "layer east local-rule in force",
                "layer west local-rule in force",
                "layer what-if emergency-declaration in force",
            ],
        ),
    ];
    for (names, c, want) in cases {
        let m = members(names);
        assert_eq!(reference(&m, c), want, "oracle, {names:?}");
        assert_eq!(compiled(&db, ws, &m, c), want, "compiler, {names:?}");
    }
}

/// Regression pin (found by the oracle): a precedent two valid overrulings
/// withdraw is withdrawn from the earlier, en banc in 2015, whatever the
/// overrulings are named. The compiler kept the overruling whose package
/// name sorted last, the Supreme Court's of 2018, so a rename moved the
/// withdrawal by three years.
#[test]
fn two_overrulings_withdraw_from_the_earlier() {
    let (db, ws) = change_set();
    let m = members(&["ca1-rule", "en-banc", "scotus-rev"]);
    let want = [
        "layer ca1-rule binding precedent not in force (overruled by en-banc from 2015-03-01)",
        "layer en-banc overruling in force",
        "layer scotus-rev overruling in force",
    ];
    assert_eq!(compiled(&db, ws, &m, AT_CA1), want);
    assert_eq!(reference(&m, AT_CA1), want);
}

/// Regression pin (found by the oracle): a precedent that does not bind at
/// the world's location has no claim in force there, so its interpretation
/// does not conflict with the Supreme Court's. The compiler merged every
/// layer for the conflict check and reported L3005 at `ca2`.
#[test]
fn a_precedent_that_does_not_bind_here_conflicts_with_nothing() {
    let (db, ws) = change_set();
    let m = members(&["ca1-rule", "high-rule"]);
    let ca2 = Coordinates {
        circuit: "ca2",
        ..AT_CA1
    };
    let want = [
        "L9009 ca1-rule",
        "layer ca1-rule persuasive precedent not in force",
        "layer high-rule binding precedent in force",
    ];
    assert_eq!(compiled(&db, ws, &m, ca2), want);
    assert_eq!(reference(&m, ca2), want);
}
