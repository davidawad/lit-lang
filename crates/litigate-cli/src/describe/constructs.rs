//! Every construct with an example (DESIGN §8 `describe`). Each `.lit`
//! example is a whole item and parses inside a module; each `.case`
//! snippet is a line of [`CASE_EXAMPLE`], which parses (tests below).
use super::Construct;

/// A complete matter; the `.case` construct snippets are drawn from it.
pub const CASE_EXAMPLE: &str = r#"case acme-v-widget
uses demo-rules ^0.1.0
as_of 2025-06-01
known_at 2025-06-01
summary "Acme sues Widget; Widget decides whether to answer or move to dismiss."
notes "Stakes are illustrative."
source "Demo Rules of Procedure" { url "https://example.org/rules", as_of 2024-12-01 }
perspective widget
parties {
  acme: Plaintiff Corporation
  widget: Defendant Corporation
}
events {
  2025-03-05 Served(complaint, Personal) on widget
  2025-03-20 MovedToDismiss(complaint) by widget
}
facts { time-barred: not-barred }
stakes { judgment-entered: -2_000_000.usd, dismissed: 0.usd }
beliefs { observe mtd-ruling: { granted: 3, denied: 1 } }
analysis { params: { rate: 650, opp_rate: 550 } }
"#;

const LIT: [(&str, &str, &str, &str); 26] = [
    ("edition", "Optional first line of a .lit or .case file naming its language edition; absent means 2026 (L0008 for an unknown year).", "edition 2026", "not emitted"),
    ("module", "Names the module; the first line of every .lit file.", "module demo.rules", "not emitted"),
    ("import", "Brings another module's declarations into scope.", "import demo.counting.{rule6}", "not emitted"),
    ("package", "Optional restatement of Lit.toml identity (must agree, L1005).", "package demo-rules { forum demo, authority FederalRule }", "not emitted"),
    ("start", "The package's start node (exactly one per package, L1008).", "start time-barred", "Pack.startNodeId"),
    ("group", "A display group for nodes.", "group pleadings: \"Pleadings\"", "Pack.groups[]"),
    ("type", "A type alias.", "type Stake = Money<usd>", "not emitted"),
    ("enum", "A closed sum of variants; matches over it must be exhaustive.", "enum ServiceMethod { Personal, Mail, Electronic }", "not emitted"),
    ("event", "An event type: what can happen in a matter, and what edges and deadlines match. One optional effect, `removes p`, `elects p`, `extends d to p`, or `waives d`, is the only way an event acts besides taking an edge or firing a trigger; names mean nothing.", "event AnswerExtended(until: Date) extends answer-period to until", "not emitted"),
    ("state", "A node with no single mover.", "state answered: \"Answer filed\" { group pleadings, _ \"Proceed to judgment\" -> judgment-entered }", "Pack.nodes[] kind state"),
    ("decision", "A node where one role chooses among edges.", "decision respond: \"Answer or move\" by Defendant cite R. 12(a)(1)(A)(i)\n{\n  answer \"File an answer\" -> answered { event Answered(complaint), effort 8.hours, within answer-period }\n  default \"Let the deadline pass\" -> default-risk { sets [defaulted], valence bad }\n}", "Pack.nodes[] kind decision, Pack.edges[]"),
    ("chance", "A node resolved by nature or a court's discretion; probabilities must sum to exactly 1 (L2003) or be holes.", "chance mtd-ruling: \"Ruling\" by Court discretion cite R. 12(b)(6)\n{\n  granted \"Granted\" -> dismissed p 1/4\n  denied \"Denied\" -> answered p ?(\"no calibration yet\")\n}", "Pack.nodes[] kind state, edges with probability"),
    ("terminal", "An absorbing outcome node.", "terminal dismissed: \"Action dismissed\" outcome [judgment, win] { payoff 0.usd }", "Pack.nodes[] kind terminal"),
    ("edge", "An edge inside a node body: id (or `_`), label, target, and fields.", "state s: \"S\" { go \"Go\" -> t { cost 1_500.usd, effort 2.hours, tags [x] } }", "Pack.edges[]"),
    ("link", "A cross-package edge (links packages only).", "link appeal \"Notice of appeal\": trial::judgment -> appeal::notice-filed by Plaintiff", "LinkFile.links[]"),
    ("instance", "A re-rooted copy of a pack for a link origin.", "instance appeal@trial { pack appeal, remove_edges [x], roles { appellant: self } }", "LinkFile.instances[]"),
    ("rule", "A duty, power, or liability rule grouping deadlines.", "rule duty answer-duty cite R. 12(a)(1)(A)(i)\n{\n  deadline answer-period: 21.days from Served(complaint) counted_by rule6\n}", "not emitted (verification); its deadlines lower to edge deadlines"),
    ("deadline", "A dated, counted period from a trigger event. Each trigger creates an obligation instance: `bearer` names the actor bearing it, `trigger first|latest|each|later_of(d)` what a repeated trigger does, `extend` grants (`max N.days` or a date bound `to max(prescribed_due + 30.days, grant_order_entered + 14.days)`, with an optional standard and motion `on M() [, within W]`; `discretion S [, after_expiry] [, on M()]` with S one of GoodCause, ExcusableNeglect, GoodCauseOrExcusableNeglect (before expiry with `on M()`, the window is the request's date, not the order's); or the lock `never cite X`) and `waive` the capabilities that transform it. Linear in a `rule duty`.", "deadline answer-period: 21.days from Served(complaint) counted_by rule6 effective [2009-12-01, ..) delegable_to [CaseOrder] { bearer Defendant, extend { by Court, discretion GoodCause }, extend { by Court, discretion GoodCause, on ExtensionMoved() }, extend { by Court, discretion ExcusableNeglect, after_expiry, on ExtensionMoved() } }", "RawEdge.deadline via `within`"),
    ("counting", "A deadline counting rule (R. 6 style).", "counting rule6 cite R. 6(a)\n{\n  exclude_trigger_day\n  include_last_day\n  roll_forward_on [weekend, legal_holiday, inaccessible]\n  service_extension { Mail: 3.days } cite R. 6(d)\n  holidays court-holidays\n  anchor trigger_day\n}", "clock library countingRules[]"),
    ("calendar", "A holiday calendar.", "calendar court-holidays cite R. 6(a)(6)\n{\n  juneteenth: fixed(6, 19) effective [2021-06-17, ..) observed nearest_weekday\n}", "clock library calendars[]"),
    ("transition", "Which version governs a matter that straddles an amendment.", "transition time-2009 amends answer-period@2009-12-01 cite R. 86(a)(2)(B)\n{\n  unless has_event(OrderedPriorRules()) -> answer-period@2007-12-01\n  otherwise -> answer-period@2009-12-01\n}", "not emitted (verification)"),
    ("authority", "The authority lattice, highest first.", "authority Statute > FederalRule > LocalRule > StandingOrder > CaseOrder", "not emitted (verification)"),
    ("default", "A default rule (Catala-style) with a condition.", "default standing when has_event(Served(complaint, Mail)) => 14.days", "not emitted (verification)"),
    ("exception", "A rebutting or undercutting exception to a default.", "exception local rebuts standing when has_event(Answered(complaint)) => 7.days priority 2", "not emitted (verification)"),
    ("test", "An inline test: given events, expected values (`litigate test`).", "test \"answer due 21 days after personal service\" cite R. 12(a)(1)(A)(i)\n{\n  given Served(complaint, Personal) on 2025-03-03\n  expect deadline(answer-period) == 2025-03-24\n}", "not emitted (tests-pass obligation)"),
    ("version", "Effective intervals, corpus dates, and relations on any rule-bearing declaration.", "deadline d: 20.days from Served(complaint) counted_by rule6 effective [2007-12-01, 2009-12-01) known_at 2007-04-30 supersedes d@1993-12-01", "only the version selected by --as-of/--known-at is emitted"),
];

const CASE: [(&str, &str, &str, &str); 15] = [
    ("case", "Names the matter; the first line of every .case file.", "case acme-v-widget", "NamedScenario.id"),
    ("uses", "Pins the packages it uses; the first is primary.", "uses demo-rules ^0.1.0", "NamedScenario.packs"),
    ("as_of", "Selects the law in force (compile time only).", "as_of 2025-06-01", "provenance only"),
    ("known_at", "Selects the corpus date.", "known_at 2025-06-01", "provenance only"),
    ("summary", "One-line summary.", "summary \"Acme sues Widget; Widget decides whether to answer or move to dismiss.\"", "NamedScenario.summary"),
    ("notes", "Free-text notes.", "notes \"Stakes are illustrative.\"", "NamedScenario.notes"),
    ("source", "A sourced fact behind the matter's numbers.", "source \"Demo Rules of Procedure\" { url \"https://example.org/rules\", as_of 2024-12-01 }", "NamedScenario.sources[]"),
    ("perspective", "Whose side the analysis takes (a party, or an actor-to-role map).", "perspective widget", "scenario.perspective"),
    ("parties", "Each party and its role in the primary package.", "  acme: Plaintiff Corporation", "provenance only"),
    ("events", "Dated history, replayed to derive posture (never asserted).", "  2025-03-05 Served(complaint, Personal) on widget", "scenario.start (replayed)"),
    ("facts", "Resolved fact nodes: node to its edge.", "facts { time-barred: not-barred }", "scenario.facts"),
    ("stakes", "Terminal values; every reachable terminal needs one (L4004).", "stakes { judgment-entered: -2_000_000.usd, dismissed: 0.usd }", "scenario.payoffs"),
    ("beliefs", "Observed outcome counts at chance nodes.", "beliefs { observe mtd-ruling: { granted: 3, denied: 1 } }", "scenario.observe"),
    ("analysis", "Any other litgraph Scenario field, as JSON.", "analysis { params: { rate: 650, opp_rate: 550 } }", "scenario.<field>"),
    ("start", "An asserted start (legacy imports); must agree with replay (L4008).", "start demo-rules::mtd-ruling", "scenario.start"),
];

/// Every construct, `.lit` first.
#[must_use]
pub fn all() -> Vec<Construct> {
    let lit = LIT.iter().map(|(n, s, e, l)| (*n, "lit", *s, *e, *l));
    let case = CASE.iter().map(|(n, s, e, l)| (*n, "case", *s, *e, *l));
    lit.chain(case)
        .map(|(name, file, summary, example, lowers_to)| Construct {
            name: name.to_owned(),
            file: file.to_owned(),
            summary: summary.to_owned(),
            example: example.trim_start().to_owned(),
            lowers_to: lowers_to.to_owned(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use litigate_syntax::{parse, FileKind};

    #[test]
    fn lit_examples_parse() {
        for (name, _, example, _) in LIT {
            let text = if name == "module" {
                format!("{example}\n")
            } else if name == "edition" {
                format!("{example}\nmodule m\n")
            } else {
                format!("module m\n\n{example}\n")
            };
            let parsed = parse(&text, FileKind::Lit);
            assert!(parsed.is_ok(), "{name}: {:?}", parsed.errors());
        }
    }

    #[test]
    fn case_snippets_come_from_a_parsing_matter() {
        let parsed = parse(CASE_EXAMPLE, FileKind::Case);
        assert!(parsed.is_ok(), "{:?}", parsed.errors());
        for (name, _, snippet, _) in CASE.iter().filter(|c| c.0 != "start") {
            assert!(CASE_EXAMPLE.contains(snippet), "{name}");
        }
    }
}
