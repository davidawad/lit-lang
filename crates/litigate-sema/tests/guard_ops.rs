//! Typed guard operations in patches (SPEC §3.6): `when and` narrows a
//! patched fact edge's guard, `when or` widens it, `supersedes when`
//! replaces it, each on its instrument's authority. Replay composes the
//! operations in force on the date it reads, in the order authority and
//! effective date fix (never source order), and the trace records the
//! operations behind a derived fact. `check` rejects an operation off a
//! fact edge (L2007), without a capability (L3004), unordered against one
//! it does not commute with (L3003), or one that makes sibling guards
//! overlap (L2005).
mod cov_case_support;

use cov_case_support::{check, compile, lock, matter, outcome, package, At};
use litigate_base::{Profile, Severity};
use litigate_sema::hir::Package;
use std::collections::BTreeMap;
use std::fmt::Write as _;

const BASE: &str = "module base
start intake

authority FederalRule > LocalRule > CaseOrder

event Ordered()
event Filed()
event Moved()
event Struck()
event Excused()
event Noted()

counting bc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays bcal }
calendar bcal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

deadline after-move: 30.days from Moved() counted_by bc
deadline after-strike: 30.days from Struck() counted_by bc
deadline after-excuse: 30.days from Excused() counted_by bc

state intake: \"Intake\" by Plaintiff
{
  file \"File\" -> check { event Filed() }
}

chance check: \"Timely?\" by Court delegable_to [CaseOrder]
{
  tags [fact]
  timely \"Timely\" -> heard p 1/2 { when has_event(Moved()) }
  late \"Late\" -> barred p 1/2 { when !has_event(Moved()) && has_event(Struck()) }
}

state heard: \"Heard\" by Plaintiff
{
  go \"Go\" -> won { event Noted() }
}

terminal won: \"Won\" { payoff 0.usd }
terminal barred: \"Barred\" { payoff 0.usd }
";

/// The change layer: a link into instance `base@order` and its patches.
fn layer(patches: &str) -> String {
    format!(
        "module order
import base.{{ Ordered, Moved, Struck, Excused, FederalRule, LocalRule, CaseOrder }}
link enter \"Enter\": base::intake -> base@order::intake by Court {{ event Ordered() }}
instance base@order {{
  pack base
{patches}
}}
"
    )
}

fn loaded(patches: &str) -> Vec<litigate_sema::pkg::load::LoadedPackage> {
    vec![
        package("base", &[], "", BASE),
        package("order", &["base"], "", &layer(patches)),
    ]
}

/// `base` and the layer with `patches`, compiled.
fn world(patches: &str) -> Vec<Package> {
    compile(&loaded(patches))
}

/// The diagnostics of the layer with `patches`, `line:col code message`.
fn diagnostics(patches: &str) -> Vec<String> {
    let mut db = litigate_sema::LitDatabase::default();
    let ws = litigate_sema::db::inputs::create_workspace(&mut db, &loaded(patches));
    litigate_sema::check_workspace(&db, ws)
        .iter()
        .flat_map(|c| c.diagnostics.iter())
        .filter(|d| d.severity == Severity::Error)
        .map(|d| {
            format!(
                "{}:{} {} {}{}",
                d.span.line,
                d.span.col,
                d.code.as_str(),
                d.message,
                d.notes.iter().fold(String::new(), |mut s, n| {
                    let _ = write!(s, "\n    note: {}", n.message);
                    s
                })
            )
        })
        .collect()
}

const ORDERED: &str = "2010-02-01 Ordered() by c";
const MOVED: &str = "2010-02-02 Moved() by p";
const STRUCK: &str = "2010-02-03 Struck() by d";
const EXCUSED: &str = "2010-02-04 Excused() by c";
const FILED: &str = "2010-02-05 Filed() by p";
const NOTED: &str = "2010-02-06 Noted() by p";

/// The outcome of a matter over `packages` with `events`, then the facts
/// it lowers and the guard operations each fact step records.
fn replay(packages: &[Package], events: &[&str]) -> Vec<String> {
    replay_known(packages, events, None)
}

/// [`replay`] with the matter's corpus date `known_at`.
fn replay_known(packages: &[Package], events: &[&str], known_at: Option<&str>) -> Vec<String> {
    let mut text = matter(
        "base ^1.0.0, order ^1.0.0",
        "",
        events,
        "stakes { base::won: 0.usd, base::barred: 0.usd }",
    );
    if let Some(k) = known_at {
        text = text.replace(
            "as_of 2011-06-01\n",
            &format!("as_of 2011-06-01\nknown_at {k}\n"),
        );
    }
    let checked = check(
        &text,
        packages,
        Some(&lock(packages)),
        &At("2010-03-30"),
        Profile::Dev,
    );
    let mut out = outcome(&checked);
    out.extend(checked.trace.iter().flat_map(|s| {
        s.guard_ops
            .iter()
            .map(|g| serde_json::to_string(g).unwrap_or_default())
    }));
    out.extend(
        checked
            .case
            .iter()
            .flat_map(|c| c.facts.iter())
            .map(|(n, e)| format!("fact {n} = {e}")),
    );
    out
}

const NARROW: &str = "  patch timely {
    when and has_event(Excused()) by FederalRule effective [2010-01-01, ..) cite \"Order, Dkt. 12\"
  }";

const WIDEN: &str = "  patch timely {
    when or has_event(Excused()) by FederalRule effective [2010-01-01, ..) cite \"Order, Dkt. 12\"
  }
  patch late {
    when and !has_event(Excused()) by FederalRule effective [2010-01-01, ..) cite \"Order, Dkt. 12\"
  }";

const REPLACE: &str = "  patch timely {
    supersedes when has_event(Excused()) by FederalRule effective [2010-01-01, ..) cite \"Order, Dkt. 12\"
  }
  patch late {
    supersedes when !has_event(Excused()) && has_event(Struck()) by FederalRule effective [2010-01-01, ..) cite \"Order, Dkt. 12\"
  }";

/// Two instruments on one edge: a case order (delegated) replacing the
/// guard, and an earlier-dated federal rule narrowing it. Authority
/// orders them (the case order first), whatever the dates and the source
/// order say.
const LAYERED: &str = "  patch timely {
    when and has_event(Excused()) by FederalRule effective [2009-12-01, ..) cite \"Fed. R. Civ. P. 1\"
    supersedes when has_event(Struck()) by CaseOrder effective [2010-01-15, ..) cite \"Order, Dkt. 12\"
  }
  patch late {
    when and !has_event(Excused()) by FederalRule effective [2009-12-01, ..) cite \"Fed. R. Civ. P. 1\"
  }";

const LAYERED_REVERSED: &str = "  patch late {
    when and !has_event(Excused()) by FederalRule effective [2009-12-01, ..) cite \"Fed. R. Civ. P. 1\"
  }
  patch timely {
    supersedes when has_event(Struck()) by CaseOrder effective [2010-01-15, ..) cite \"Order, Dkt. 12\"
    when and has_event(Excused()) by FederalRule effective [2009-12-01, ..) cite \"Fed. R. Civ. P. 1\"
  }";

#[test]
fn guard_operations_in_replay() {
    let base = world("");
    let narrow = world(NARROW);
    let widen = world(WIDEN);
    let replace = world(REPLACE);
    let later = world(&NARROW.replace("2010-01-01", "2012-01-01"));
    let hole = world(&NARROW.replace("cite \"Order, Dkt. 12\"", "cite ?(\"Order, Dkt. 12\")"));
    let ended = world(&NARROW.replace("[2010-01-01, ..)", "[2010-01-01, 2010-02-05)"));
    let recorded =
        world(&NARROW.replace("[2010-01-01, ..)", "[2010-01-01, ..) known_at 2025-09-01"));
    let rows: BTreeMap<&str, Vec<String>> = [
        (
            "base: the unpatched guard derives timely",
            replay(&base, &[ORDERED, MOVED, FILED]),
        ),
        (
            "narrow: `g && C` no longer holds without the excuse",
            replay(&narrow, &[ORDERED, MOVED, FILED]),
        ),
        (
            "narrow: `g && C` holds with it, and the trace names the operation",
            replay(&narrow, &[ORDERED, MOVED, EXCUSED, FILED, NOTED]),
        ),
        (
            "widen: `g || C` holds on the excuse alone",
            replay(&widen, &[ORDERED, EXCUSED, FILED]),
        ),
        (
            "widen: the narrowed sibling no longer derives late on an excuse",
            replay(&widen, &[ORDERED, STRUCK, EXCUSED, FILED]),
        ),
        (
            "replace: `C` alone, the base guard is gone",
            replay(&replace, &[ORDERED, MOVED, FILED]),
        ),
        (
            "replace: `C` holds",
            replay(&replace, &[ORDERED, EXCUSED, FILED]),
        ),
        (
            "not yet in force: the base guard holds",
            replay(&later, &[ORDERED, MOVED, FILED]),
        ),
        (
            "ended before the date read: the base guard holds",
            replay(&ended, &[ORDERED, MOVED, FILED, NOTED]),
        ),
        (
            "known_at: recorded after the matter's corpus date, not yet known",
            replay_known(&recorded, &[ORDERED, MOVED, FILED], Some("2025-06-01")),
        ),
        (
            "known_at: recorded by the matter's corpus date, it applies",
            replay_known(&recorded, &[ORDERED, MOVED, FILED], Some("2025-10-01")),
        ),
        (
            "cite hole: unverified, the operation does not apply",
            replay(&hole, &[ORDERED, MOVED, FILED]),
        ),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}

/// Two layers composing: the case order's replacement applies before the
/// federal rule's narrowing (lower authority first), so the guard is
/// `Struck && Excused` in either source order; a struck filing without an
/// excuse falls to the narrowed `late`.
#[test]
fn layered_operations_compose_in_authority_order_not_source_order() {
    let layered = world(LAYERED);
    let reversed = world(LAYERED_REVERSED);
    for events in [
        &[ORDERED, STRUCK, EXCUSED, FILED, NOTED][..],
        &[ORDERED, STRUCK, FILED],
        &[ORDERED, MOVED, EXCUSED, FILED],
    ] {
        assert_eq!(replay(&layered, events), replay(&reversed, events));
    }
    insta::assert_debug_snapshot!([
        replay(&layered, &[ORDERED, STRUCK, EXCUSED, FILED, NOTED]),
        replay(&layered, &[ORDERED, STRUCK, FILED]),
        replay(&layered, &[ORDERED, MOVED, EXCUSED, FILED]),
    ]);
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per rule
fn guard_operation_checks() {
    // Narrowing `timely` (or replacing it inside `has_event(Moved())`)
    // keeps it disjoint from `late`, whose guard needs `!has_event(Moved())`.
    let op = |kind: &str, by: &str, from: &str| {
        format!(
            "{kind} has_event(Moved()) && has_event(Excused()) by {by} effective [{from}, ..) \
             cite \"Order, Dkt. 12\""
        )
    };
    let rows: BTreeMap<&str, Vec<String>> = [
        (
            "accepted: narrow, widen and replace with their authority",
            diagnostics(NARROW)
                .into_iter()
                .chain(diagnostics(WIDEN))
                .chain(diagnostics(REPLACE))
                .chain(diagnostics(LAYERED))
                .collect(),
        ),
        (
            "placement: `file` is not a fact edge, so it carries no guard (L2007)",
            diagnostics(&format!(
                "  patch file {{ {} }}",
                op("when and", "FederalRule", "2010-01-01")
            )),
        ),
        (
            "capability: a local rule the node does not delegate to (L3004)",
            diagnostics(&format!(
                "  patch timely {{ {} }}",
                op("supersedes when", "LocalRule", "2010-01-01")
            )),
        ),
        (
            "capability: a delegated case order is licensed",
            diagnostics(&format!(
                "  patch timely {{ {} }}",
                op("when and", "CaseOrder", "2010-01-01")
            )),
        ),
        (
            "authority: a `cite` or an `effective` interval missing (L2007)",
            diagnostics(&format!(
                "  patch timely {{\n    {}\n    {}\n  }}",
                op("when and", "FederalRule", "2010-01-01").replace(" cite \"Order, Dkt. 12\"", ""),
                op("when and", "FederalRule", "2010-01-01")
                    .replace(" effective [2010-01-01, ..)", "")
            )),
        ),
        (
            "authority: an empty effective interval",
            diagnostics(&format!(
                "  patch timely {{ {} }}",
                op("when and", "FederalRule", "2010-01-01").replace("..)", "2009-01-01)")
            )),
        ),
        (
            "unknown level (L1002)",
            diagnostics(&format!(
                "  patch timely {{ {} }}",
                op("when and", "Nope", "2010-01-01")
            )),
        ),
        (
            "order: same authority and date, replace and narrow do not commute (L3003)",
            diagnostics(&format!(
                "  patch timely {{\n    {}\n    {}\n  }}",
                op("supersedes when", "FederalRule", "2010-01-01"),
                op("when and", "FederalRule", "2010-01-01")
            )),
        ),
        (
            "order: two narrowings commute, so no order is needed",
            diagnostics(&format!(
                "  patch timely {{\n    {}\n    {}\n  }}",
                op("when and", "FederalRule", "2010-01-01"),
                op("when and", "FederalRule", "2010-01-01")
            )),
        ),
        (
            "order: disjoint intervals never meet",
            diagnostics(&format!(
                "  patch timely {{\n    {}\n    {}\n  }}",
                op("supersedes when", "FederalRule", "2010-01-01").replace("..)", "2010-06-01)"),
                op("when and", "FederalRule", "2010-06-01")
            )),
        ),
        (
            "disjointness: widening timely lets it overlap late, reported once (L2005)",
            diagnostics(&format!(
                "  patch timely {{\n    {}\n    {}\n  }}",
                op("when or", "FederalRule", "2010-01-01").replace("has_event(Moved()) && ", ""),
                op("when or", "FederalRule", "2010-06-01").replace("has_event(Moved()) && ", "")
            )),
        ),
    ]
    .into_iter()
    .collect();
    insta::assert_debug_snapshot!(rows);
}

/// A guard operation the parser could not finish (no condition) is left
/// out of the item tree; the parse error stands alone.
#[test]
fn a_guard_operation_without_a_condition_is_dropped() {
    let parsed = litigate_sema::db::parse::parse(&layer("  patch timely {\n    when and\n  }"));
    let codes: Vec<&str> = parsed.reports.iter().map(|r| r.code.as_str()).collect();
    assert_eq!(codes, ["L0001"]);
}
