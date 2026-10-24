//! Precedence selects (SPEC §5.5; design review 2026-10-23 §5.1 P1, P2, P5,
//! P6): one query picks the declaration that governs, and case replay
//! judges every event by it. One negative test per defect: a displacing
//! claim nothing licenses has no effect, incomparable authorities are an
//! error, and a shorter window alone never wins.
mod cov_case_support;
mod legal;

use cov_case_support::{check, compile, lock, outcome};
use legal::{pkg, render, workspace, CORE};
use litigate_base::Profile;
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::{Bound, DateRef, Declaration, Extension, Package, Relation};
use litigate_sema::pkg::load::LoadedPackage;
use litigate_sema::precedence::select::{resolve, unresolved};
use litigate_sema::precedence::{conclude, select, Conclusion, Unselected};
use litigate_sema::temporal::world::nowhere;
use litigate_sema::temporal::{Inactive, Key, Miss, World};
use std::cell::RefCell;
use std::fmt::Write as _;

const START: &str = "module core.start\nstart z\nstate z: \"Z\"\n";

fn core() -> LoadedPackage {
    let chain = format!("{CORE}authority Statute > Regulation > CaseOrder\n");
    pkg(
        "core",
        "Statute",
        "1990-01-01",
        &[],
        &[("authority.lit", &chain), ("start.lit", START)],
    )
}

/// `fed` (`FederalRule`): `answer-period`, whose `clauses` decide whom it
/// delegates to, taken `within` by `answer`.
fn fed(clauses: &str) -> LoadedPackage {
    fed_with(clauses, "")
}

/// [`fed`] with `counting` clauses on its counting rule `c6`.
fn fed_with(clauses: &str, counting: &str) -> LoadedPackage {
    let text = format!(
        "module fed.rules
import core.authority.{{Statute, LocalRule, StandingOrder, CaseOrder, Regulation}}
start s
enum Document {{ complaint, motion }}
event Served(document: Document)
event Answered(document: Document)
counting c6 {counting} {{ holidays cal }}
calendar cal {{ new-year: fixed(1, 1) effective [1870-06-28, ..) }}
deadline answer-period: 21.days from Served() counted_by c6 effective [2009-12-01, ..){clauses}
state s: \"S\" by Plaintiff {{ serve \"Serve\" -> r {{ event Served(complaint) }} }}
decision r: \"R\" by Defendant {{ answer \"Answer\" -> done {{ event Answered(complaint), within answer-period }} }}
terminal done: \"Done\" {{ payoff 0.usd }}
"
    );
    pkg(
        "fed",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[("rules.lit", &text)],
    )
}

/// A package `name` of `authority` over `fed` declaring `body`.
fn over(name: &str, authority: &str, body: &str) -> LoadedPackage {
    let text = format!(
        "module {name}.rules
import fed.rules.{{answer-period, c6, Served, Document}}
start l
state l: \"L\"
{body}
"
    );
    pkg(
        name,
        authority,
        "2009-12-01",
        &["core", "fed"],
        &[("rules.lit", &text)],
    )
}

/// A deadline `id` of `days` from `trigger` with `clauses`.
fn deadline(id: &str, days: u32, trigger: &str, clauses: &str) -> String {
    format!("deadline {id}: {days}.days from {trigger} counted_by c6 {clauses}\n")
}

fn key(pack: &str, name: &str) -> Key {
    Key {
        pack: pack.into(),
        name: name.into(),
    }
}

/// Every diagnostic of `packages` as written.
fn diagnostics(packages: &[LoadedPackage]) -> Vec<String> {
    let (db, ws) = workspace(packages);
    render(&db, ws)
}

/// `hir` with every delegation to `level` of package `fed` removed: what
/// selection sees if an unlicensed claim reached it (`check` rejects the
/// source with L3004, so only HIR built directly can carry one).
fn undelegated(mut hir: Vec<Package>, level: &str) -> Vec<Package> {
    for p in hir.iter_mut().filter(|p| p.manifest.name == "fed") {
        for d in p.modules.iter_mut().flat_map(|m| m.declarations.iter_mut()) {
            if let Declaration::Deadline(x) = d {
                x.meta.delegations.retain(|g| g.to != level);
            }
        }
    }
    hir
}

/// Who governs `fed::answer-period` on 2012-06-01 among `hir`.
fn who(hir: &[Package]) -> String {
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    match select(&world, &key("fed", "answer-period"), "2012-06-01", None) {
        Ok(w) => format!("{}::{}", world.pack_of(w), world.decls[w].meta.id),
        Err(Unselected::Unresolved(pairs)) => format!("unresolved {}", pairs.len()),
        Err(Unselected::Inactive(_)) => "inactive".to_owned(),
    }
}

const DELEGATED: &str = " delegable { to LocalRule, op set }";

#[test]
fn p1_a_licensed_override_is_selected() {
    let packages = [
        core(),
        fed(DELEGATED),
        over(
            "local",
            "LocalRule",
            &deadline("local-answer", 14, "Served()", "override answer-period"),
        ),
    ];
    assert_eq!(who(&compile(&packages)), "local::local-answer");
    assert_eq!(
        who(&undelegated(compile(&packages), "LocalRule")),
        "fed::answer-period"
    );
}

#[test]
fn p1_a_displacing_claimant_is_followed_to_its_own_winner() {
    let standing = pkg(
        "standing",
        "StandingOrder",
        "2009-12-01",
        &["core", "fed", "local"],
        &[(
            "rules.lit",
            "module standing.rules\nimport fed.rules.{c6, Served}\nimport local.rules.{local-answer}\nstart m\nstate m: \"M\"\ndeadline standing-answer: 10.days from Served() counted_by c6 override local-answer\n",
        )],
    );
    let local = pkg(
        "local",
        "LocalRule",
        "2009-12-01",
        &["core", "fed"],
        &[(
            "rules.lit",
            "module local.rules\nimport core.authority.{StandingOrder}\nimport fed.rules.{answer-period, c6, Served}\nstart l\nstate l: \"L\"\ndeadline local-answer: 14.days from Served() counted_by c6 override answer-period delegable { to StandingOrder, op set }\n",
        )],
    );
    let hir = compile(&[core(), fed(DELEGATED), local, standing]);
    assert_eq!(who(&hir), "standing::standing-answer");
}

#[test]
fn p2_every_displacing_relation_needs_a_delegation() {
    // A 3-day period over the 21-day one shortens it; a supersession of
    // another declaration suspends it.
    let rows = [
        (
            "override",
            "override answer-period",
            "order::order",
            "shorten",
        ),
        (
            "notwithstanding",
            "notwithstanding answer-period",
            "order::order",
            "shorten",
        ),
        (
            "supersedes",
            "supersedes answer-period",
            "inactive",
            "suspend",
        ),
    ];
    for (relation, clause, licensed, op) in rows {
        let decl = deadline(
            "order",
            3,
            "Served()",
            &format!("effective [2012-01-01, ..) {clause}"),
        );
        let order = over("order", "CaseOrder", &decl);
        let found = diagnostics(&[core(), fed(""), order.clone()]);
        assert_eq!(found.len(), 1, "{relation}: {found:#?}");
        assert!(
            found[0].contains("L3004")
                && found[0].contains(&format!("missing `delegable {{ to CaseOrder, op {op} }}`")),
            "{relation}: {found:#?}"
        );
        let hir = compile(&[
            core(),
            fed(&format!(" delegable {{ to CaseOrder, op {op} }}")),
            order,
        ]);
        assert_eq!(who(&hir), licensed, "{relation}");
        assert_eq!(
            who(&undelegated(hir, "CaseOrder")),
            "fed::answer-period",
            "{relation}: an unlicensed claim has no effect"
        );
    }
}

#[test]
fn p2_a_revival_needs_a_delegation_too() {
    // A local rule suspends `answer-period` from 2011; a case order revives
    // it from 2012, which only a delegation to case orders makes effective.
    let pause = over(
        "local",
        "LocalRule",
        &deadline(
            "pause",
            3,
            "Served()",
            "effective [2011-01-01, ..) supersedes answer-period",
        ),
    );
    let resume = over(
        "order",
        "CaseOrder",
        &deadline(
            "resume",
            3,
            "Served()",
            "effective [2012-01-01, ..) revives answer-period",
        ),
    );
    let suspend = " delegable { to LocalRule, op suspend }";
    let found = diagnostics(&[core(), fed(suspend), pause.clone(), resume.clone()]);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("L3004")
            && found[0].contains("`resume` (CaseOrder) revives `fed::answer-period`"),
        "{found:#?}"
    );
    let hir = compile(&[
        core(),
        fed(" delegable { to [LocalRule, CaseOrder], op suspend }"),
        pause,
        resume,
    ]);
    assert_eq!(who(&hir), "fed::answer-period");
    assert_eq!(who(&undelegated(hir, "CaseOrder")), "inactive");
}

/// A `Regulation` package `name` over `fed` (and `deps`) whose claim on
/// `answer-period` no relation orders against the local rule's.
fn agency(deps: &[&str]) -> LoadedPackage {
    let mut all = vec!["core", "fed"];
    all.extend(deps);
    let imports = if deps.is_empty() {
        String::new()
    } else {
        "import local.rules.{local-answer}\n".to_owned()
    };
    pkg(
        "agency",
        "Regulation",
        "2009-12-01",
        &all,
        &[(
            "rules.lit",
            &format!("module agency.rules\nimport fed.rules.{{answer-period, c6, Served}}\n{imports}start a\nstate a: \"A\"\ndeadline agency-answer: 30.days from Served() counted_by c6 effective [2012-01-01, 2013-01-01) subject_to answer-period\n"),
        )],
    )
}

#[test]
fn p5_incomparable_authorities_are_an_error_not_a_canon() {
    let local = over(
        "local",
        "LocalRule",
        &deadline(
            "local-answer",
            14,
            "Served()",
            "effective [2011-01-01, ..) override answer-period",
        ),
    );
    // Regulation is incomparable with the court chain; neither the later
    // `from` nor the narrower window may decide it.
    let found = diagnostics(&[core(), fed(DELEGATED), local.clone(), agency(&["local"])]);
    assert!(
        found.iter().any(|d| d.contains("L3003")
            && d.contains("Regulation and LocalRule are not ranked by any `authority` chain, so no canon orders them")),
        "{found:#?}"
    );
    // Packages that do not depend on each other meet only in a matter.
    let hir = compile(&[core(), fed(DELEGATED), local, agency(&[])]);
    assert_eq!(who(&hir), "unresolved 1");
}

/// `wide` and `narrow` overrides of `answer-period` from `a` and `b`,
/// in one package (`together`) or two.
fn pair(a: &str, b: &str, together: bool) -> Vec<LoadedPackage> {
    let wide = deadline(
        "wide",
        14,
        a,
        "effective [2012-01-01, ..) override answer-period",
    );
    let narrow = deadline(
        "narrow",
        7,
        b,
        "effective [2012-01-01, 2013-01-01) override answer-period",
    );
    if together {
        vec![
            core(),
            fed(DELEGATED),
            over("local", "LocalRule", &(wide + &narrow)),
        ]
    } else {
        vec![
            core(),
            fed(DELEGATED),
            over("local", "LocalRule", &wide),
            over("other", "LocalRule", &narrow),
        ]
    }
}

#[test]
fn p6_specificity_is_subject_matter_not_window_width() {
    // Same trigger: the one-year window alone does not win.
    let found = diagnostics(&pair("Served()", "Served()", true));
    assert!(
        found
            .iter()
            .any(|d| d.contains("L3003")
                && d.contains("neither applies to a narrower subject matter")),
        "{found:#?}"
    );
    assert_eq!(
        who(&compile(&pair("Served()", "Served()", false))),
        "unresolved 1"
    );
    // A trigger that entails the other's is narrower subject matter.
    let hir = compile(&pair("Served()", "Served(complaint)", true));
    assert_eq!(who(&hir), "local::narrow");
    let hir = compile(&pair("Served(motion)", "Served()", true));
    assert_eq!(who(&hir), "local::wide");
}

/// `text` in package `fed` with the shared vocabulary.
fn program(text: &str) -> LoadedPackage {
    pkg(
        "fed",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[(
            "rules.lit",
            &format!(
                "module fed.rules\nstart s\nenum Document {{ complaint }}\nevent Served(document: Document)\nevent Answered(document: Document)\ncounting c6 {{ holidays cal }}\ncalendar cal {{ new-year: fixed(1, 1) effective [1870-06-28, ..) }}\nstate s: \"S\"\n{text}"
            ),
        )],
    )
}

#[test]
fn p6_default_conditions_order_by_entailment() {
    let defaults = "default base when has_event(Served(complaint)) => 21.days
default general notwithstanding base when has_event(Served(complaint)) || has_event(Answered(complaint)) => 30.days
default special notwithstanding base when has_event(Served(complaint)) && has_event(Answered(complaint)) => 14.days
";
    // The narrower condition prevails over the broader one.
    let hir = compile(&[core(), program(defaults)]);
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let w = select(&world, &key("fed", "base"), "2012-06-01", None);
    assert_eq!(w.map(|w| world.decls[w].meta.id.as_str()), Ok("special"));
    // A deadline and a default share no antecedent to compare.
    let mixed = format!(
        "{defaults}deadline other: 9.days from Served() counted_by c6 notwithstanding base\n"
    );
    let found = diagnostics(&[core(), program(&mixed)]);
    assert!(
        found
            .iter()
            .any(|d| d.contains("L3003") && d.contains("`other@2009-12-01`")),
        "{found:#?}"
    );
}

#[test]
fn conclusions_come_from_the_selected_default() {
    let defaults = "default base when has_event(Served(complaint)) => 21.days
default local-days notwithstanding base when has_event(Served(complaint)) => 14.days
";
    let hir = compile(&[core(), program(defaults)]);
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let got = conclude(&world, &key("fed", "base"), "2012-06-01", &mut |_| {
        Some(true)
    });
    let by = match got {
        Ok(Conclusion::Value { by, .. }) => by,
        other => panic!("{other:?}"),
    };
    assert_eq!(by, ["local-days"]);
}

#[test]
fn selection_failures_explain_themselves() {
    let hir = compile(&pair("Served()", "Served()", false));
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let t = key("fed", "answer-period");
    let at = nowhere("here");
    let pairs = match select(&world, &t, "2012-06-01", None) {
        Err(Unselected::Unresolved(pairs)) => pairs,
        other => panic!("{other:?}"),
    };
    let d = resolve(&world, &t, "2012-06-01", None, at.clone()).unwrap_err();
    assert_eq!(d.code.as_str(), "L3003");
    assert_eq!(d, unresolved(&world, &t, "2012-06-01", &pairs, at.clone()));
    assert_eq!(
        Unselected::Unresolved(pairs).describe(&world, "`answer-period`", "2012-06-01"),
        "no single declaration governs `answer-period` on 2012-06-01: `local::wide@2012-01-01` and `other::narrow@2012-01-01` both apply and neither prevails"
    );
    let before = resolve(&world, &t, "2000-01-01", None, at.clone()).unwrap_err();
    assert_eq!(before.code.as_str(), "L3007");
    let w = resolve(&world, &t, "2011-06-01", None, at);
    assert_eq!(
        w.map(|w| world.decls[w].meta.id.as_str()),
        Ok("answer-period")
    );
    let inactive = select(&world, &t, "2000-01-01", None).unwrap_err();
    assert!(matches!(inactive, Unselected::Inactive(Inactive::Miss(_))));
    assert_eq!(
        inactive.describe(&world, "`answer-period`", "2000-01-01"),
        "`answer-period` is not encoded before 2009-12-01: no version is in force on 2000-01-01"
    );
    // Inside the package's corpus a miss is a finding, not a hole.
    assert_eq!(
        Unselected::Inactive(Inactive::Miss(Miss::NotInForce { hidden: Vec::new() })).describe(
            &world,
            "`answer-period`",
            "2030-01-01"
        ),
        "no version of `answer-period` is in force on 2030-01-01"
    );
}

#[test]
fn a_cycle_of_claims_stops_where_it_closes() {
    let decls = "deadline a: 1.days from Served() counted_by c6\ndeadline b: 2.days from Served() counted_by c6\ndeadline c: 3.days from Served() counted_by c6\n";
    let mut hir = compile(&[core(), program(decls)]);
    // `a` override `b` override `c` override `a` (L3006 in source).
    for d in hir[1]
        .modules
        .iter_mut()
        .flat_map(|m| m.declarations.iter_mut())
    {
        if let Declaration::Deadline(x) = d {
            let target = match x.meta.id.as_str() {
                "a" => "b",
                "b" => "c",
                _ => "a",
            };
            x.meta.relations.push(Relation::Override(target.into()));
        }
    }
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let w = select(&world, &key("fed", "a"), "2012-06-01", None);
    assert_eq!(w.map(|w| world.decls[w].meta.id.as_str()), Ok("a"));
}

/// A clock whose due date encodes the period: day `amount` of February
/// 2012. It records each request.
#[derive(Default)]
struct Encoded(RefCell<Vec<DueRequest>>);

impl Clock for Encoded {
    fn due(&self, r: &DueRequest) -> Result<String, String> {
        self.0.borrow_mut().push(r.clone());
        Ok(format!("2012-02-{:02}", r.amount))
    }

    fn nominal(&self, r: &DueRequest) -> Result<String, String> {
        self.due(r)
    }
}

fn matter(uses: &str, events: &[&str]) -> String {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    format!(
        "case t\nuses {uses}\nas_of 2012-06-01\nsummary \"s\"\n\
         parties {{\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n}}\n\
         events {{\n{events}}}\nstakes {{ done: 0.usd }}\n"
    )
}

/// Replays `events` over `hir` (the matter `uses` some of them) with the
/// encoding clock.
fn replay(hir: &[Package], uses: &str, events: &[&str]) -> (Vec<String>, Vec<DueRequest>) {
    let clock = Encoded::default();
    let lock = lock(hir);
    let checked = check(
        &matter(uses, events),
        hir,
        Some(&lock),
        &clock,
        Profile::Dev,
    );
    (outcome(&checked), clock.0.into_inner())
}

const SERVED: &str = "2012-01-03 Served(complaint) on d";
const ANSWERED: &str = "2012-02-18 Answered(complaint) by d";
const BOTH: &str = "fed ^1.0.0, local ^1.0.0";

#[test]
fn p1_replay_judges_by_the_selected_claimant() {
    let local = over(
        "local",
        "LocalRule",
        &deadline("local-answer", 14, "Served()", "override answer-period"),
    );
    let hir = compile(&[core(), fed(DELEGATED), local]);
    // Answer-period's own 21 days would make 02-18 timely; the local rule
    // that displaces it allows 14.
    let (out, requests) = replay(&hir, BOTH, &[SERVED, ANSWERED]);
    assert!(
        out.iter().any(|l| l.contains("L4002")
            && l.contains("missed deadline `answer-period`, due 2012-02-14")),
        "{out:#?}"
    );
    assert!(
        out.iter()
            .any(|l| l.contains("`local-answer` runs 14 calendar days")),
        "{out:#?}"
    );
    assert_eq!(requests.last().map(|r| r.counting.as_str()), Some("c6"));
    // An unlicensed override has no effect.
    let (out, _) = replay(
        &undelegated(hir.clone(), "LocalRule"),
        BOTH,
        &[SERVED, ANSWERED],
    );
    assert!(!out.iter().any(|l| l.contains("L4002")), "{out:#?}");
    // A matter that does not use the local package is not bound by it.
    let (out, _) = replay(&hir, "fed ^1.0.0", &[SERVED, ANSWERED]);
    assert!(!out.iter().any(|l| l.contains("L4002")), "{out:#?}");
}

/// The deadlines of the obligation instances replay opens over `hir`.
fn instances(hir: &[Package]) -> Vec<String> {
    let lock = lock(hir);
    let checked = check(
        &matter(BOTH, &[SERVED, ANSWERED]),
        hir,
        Some(&lock),
        &Encoded::default(),
        Profile::Dev,
    );
    checked
        .obligations
        .iter()
        .map(|o| format!("{} {:?}", o.deadline, o.status))
        .collect()
}

#[test]
fn a_displacing_claimant_opens_no_instance_of_its_own() {
    let local = over(
        "local",
        "LocalRule",
        &format!(
            "{}{}",
            deadline("local-answer", 14, "Served()", "override answer-period"),
            deadline("local-reply", 3, "Served()", "")
        ),
    );
    let hir = compile(&[core(), fed(DELEGATED), local]);
    // `local-answer` displaces `answer-period`: the one instance, counted
    // under it, is the one `answer` discharges (late: 14 days, not 21).
    assert_eq!(
        instances(&hir),
        ["fed::answer-period Late", "local::local-reply Lapsed"]
    );
    // An unlicensed override displaces nothing: each runs on its own.
    assert_eq!(
        instances(&undelegated(hir, "LocalRule")),
        [
            "fed::answer-period Discharged",
            "local::local-answer Lapsed",
            "local::local-reply Lapsed"
        ]
    );
}

#[test]
fn p1_replay_counts_under_the_selected_counting_rule() {
    let local = over(
        "local",
        "LocalRule",
        "counting l6 override c6 { holidays lcal }\ncalendar lcal { new-year: fixed(1, 1) effective [1870-06-28, ..) }\n",
    );
    let hir = compile(&[
        core(),
        fed_with("", "delegable { to LocalRule, op compute_method }"),
        local,
    ]);
    let (_, requests) = replay(&hir, BOTH, &[SERVED, ANSWERED]);
    assert_eq!(requests.last().map(|r| r.counting.as_str()), Some("l6"));
}

/// `fed` with stage `r` and `extra` declarations.
fn stages(extra: &str) -> LoadedPackage {
    let text = format!(
        "module fed.rules
start s
enum Document {{ complaint }}
event Served(document: Document)
state s: \"S\" by Plaintiff {{ serve \"Serve\" -> r {{ event Served(complaint) }} }}
state r: \"R\" by Defendant {{ _ \"End\" -> done }}
terminal done: \"Done\" {{ payoff 0.usd }}
{extra}
"
    );
    pkg(
        "fed",
        "FederalRule",
        "2009-12-01",
        &["core"],
        &[("rules.lit", &text)],
    )
}

/// A package `name` over `fed` whose stage claims `r`.
fn claims_stage(name: &str) -> LoadedPackage {
    pkg(
        name,
        "FederalRule",
        "2009-12-01",
        &["core", "fed"],
        &[(
            "rules.lit",
            &format!("module {name}.rules\nimport fed.rules.{{r}}\nstart {name}-r\nstate {name}-r: \"R\" by Defendant override r\n"),
        )],
    )
}

#[test]
fn replay_reports_unordered_and_displaced_stages() {
    let events = [SERVED, "2012-01-04 Served(complaint) on d"];
    // A stage another declaration displaces is not in force, as in lowering.
    let hir = compile(&[
        core(),
        stages("state r2: \"R2\" by Defendant notwithstanding r { _ \"End\" -> done }"),
    ]);
    let (out, _) = replay(&hir, "fed ^1.0.0", &events);
    assert!(
        out.iter()
            .any(|l| l.contains("L3007") && l.contains("stage `fed::r` has no version in force")),
        "{out:#?}"
    );
    // Two claimants no relation orders meet in the matter.
    let hir = compile(&[core(), stages(""), claims_stage("x"), claims_stage("y")]);
    let (out, _) = replay(&hir, "fed ^1.0.0, x ^1.0.0, y ^1.0.0", &events);
    assert!(
        out.iter()
            .any(|l| l.contains("L3003")
                && l.contains("no single declaration governs stage `fed::r`")),
        "{out:#?}"
    );
}

#[test]
fn ambiguous_and_absent_targets_explain_themselves() {
    // Two versions in force at once (L3001 in source) are ambiguous.
    let mut hir = compile(&[core(), fed("")]);
    let module = &mut hir[1].modules[0];
    let twin = module
        .declarations
        .iter()
        .find(|d| matches!(d, Declaration::Deadline(x) if x.meta.id == "answer-period"))
        .cloned()
        .unwrap();
    module.declarations.push(twin);
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let why = select(&world, &key("fed", "answer-period"), "2012-06-01", None).unwrap_err();
    assert_eq!(
        why.describe(&world, "`answer-period`", "2012-06-01"),
        "2 versions of `answer-period` are in force on 2012-06-01"
    );
    // Claimants that tie while their target is not yet in force.
    let decls = "deadline x: 1.days from Served() counted_by c6 effective [2015-01-01, ..)\ndeadline y: 2.days from Served() counted_by c6\ndeadline z: 3.days from Served() counted_by c6\n";
    let mut hir = compile(&[core(), program(decls)]);
    for d in hir[1]
        .modules
        .iter_mut()
        .flat_map(|m| m.declarations.iter_mut())
    {
        if let Declaration::Deadline(x) = d {
            if x.meta.id != "x" {
                x.meta.relations.push(Relation::SubjectTo("x".into()));
            }
        }
    }
    let refs: Vec<&Package> = hir.iter().collect();
    let world = World::from_packages(&refs);
    let got = select(&world, &key("fed", "x"), "2012-06-01", None);
    assert!(
        matches!(got, Err(Unselected::Unresolved(ref p)) if p.len() == 1),
        "{got:?}"
    );
    // In force, `x` prevails over both.
    let got = select(&world, &key("fed", "x"), "2016-06-01", None);
    assert_eq!(got.map(|w| world.decls[w].meta.id.as_str()), Ok("x"));
}

#[test]
fn a_lock_bars_a_delegated_override_from_granting_an_extension() {
    // FRAP 26(b)(1): the period delegates to a local rule, but its lock
    // forbids any extension beyond its own grant.
    let locked = " delegable { to LocalRule, op set } { extend { by Court, max 30.days }, extend { never cite \"FRAP 26(b)(1)\" } }";
    let local = |clauses: &str| {
        over(
            "local",
            "LocalRule",
            &deadline(
                "local-answer",
                14,
                "Served()",
                &format!("override answer-period {clauses}"),
            ),
        )
    };
    let widening = local("{ extend { by Court, max 60.days } }");
    let found = diagnostics(&[core(), fed(locked), widening.clone()]);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(
        found[0].contains("L3004")
            && found[0].contains("is locked: `extend { never cite \"FRAP 26(b)(1)\" }`")
            && found[0].contains("a delegation does not reach past its lock"),
        "{found:#?}"
    );
    // Selection ignores the widening claim (the source is rejected, so
    // only HIR built directly can carry one).
    let mut hir = compile(&[core(), fed(locked), local("")]);
    for p in hir.iter_mut().filter(|p| p.manifest.name == "local") {
        for d in p.modules.iter_mut().flat_map(|m| m.declarations.iter_mut()) {
            if let Declaration::Deadline(x) = d {
                x.extend = vec![Extension::Bounded {
                    by: "Court".into(),
                    until: Bound::Offset {
                        from: DateRef::PrescribedDue,
                        days: 60,
                    },
                    after_expiry: false,
                    standard: None,
                    motion: None,
                    interpretation: None,
                }];
            }
        }
    }
    assert_eq!(who(&hir), "fed::answer-period");
    // The locked version's own grant, or none, is no widening.
    for clauses in [
        "{ extend { by Court, max 30.days } }",
        "{ extend { never cite \"L.R. 6.1\" } }",
        "",
    ] {
        let narrowing = local(clauses);
        assert_eq!(
            diagnostics(&[core(), fed(locked), narrowing.clone()]),
            Vec::<String>::new(),
            "{clauses}"
        );
        assert_eq!(
            who(&compile(&[core(), fed(locked), narrowing])),
            "local::local-answer",
            "{clauses}"
        );
    }
    // Without a lock the same delegation may grant one.
    let open = " delegable { to LocalRule, op set } { extend { by Court, max 30.days } }";
    let widening = local("{ extend { by Court, max 60.days } }");
    assert_eq!(
        who(&compile(&[core(), fed(open), widening])),
        "local::local-answer"
    );
    // An authority at or above the period's own may grant past the lock.
    let amended = over(
        "fed2",
        "FederalRule",
        &deadline(
            "amended",
            14,
            "Served()",
            "override answer-period { extend { by Court, max 60.days } }",
        ),
    );
    assert_eq!(
        diagnostics(&[core(), fed(locked), amended]),
        Vec::<String>::new()
    );
}
