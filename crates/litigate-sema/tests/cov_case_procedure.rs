//! A delegation's condition and procedure in a matter (SPEC §5.5; design
//! review §2.4): `delegable { to CaseOrder, op extend, discretion S, on
//! M() }` licenses an order displacing the period only on motion, so in a
//! matter the order takes effect once the matter records `M` on or before
//! the order's date. Otherwise it has no effect there (L3004) and replay
//! keeps the delegating period; the motion itself is history, legal
//! without taking an edge.
mod cov_case_support;

use cov_case_support::{check, compile, lock, outcome, package};
use litigate_base::Profile;
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::{Declaration, Package};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

const RHO: &str = "module rho
start open

enum Doc { complaint }

authority FederalRule > CaseOrder

event Served(doc: Doc)
event Answered(doc: Doc)
event ExtensionMoved()

counting oc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays ocal }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

deadline answer: 21.days from Served(complaint) counted_by oc known_at 2009-01-01 delegable { to CaseOrder, op extend, discretion GoodCause, on ExtensionMoved() } {
  bearer Defendant
}

state open: \"Open\" known_at 2009-01-01
{
  answer \"Answer\" -> open by Defendant { event Answered(complaint), within answer }
}
";

/// A case order of 2011-02-01 giving the answer 51 days.
const RESET: &str = "module reset
import rho.{ answer, oc, Served, Doc }
start done

deadline answer-extended: 51.days from Served(complaint) counted_by oc effective [2011-02-01, ..) known_at 2011-02-01 override answer { bearer Defendant }

terminal done: \"Done\" { payoff 0.usd }
";

fn packages() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| {
        let mut order = package("reset", &["rho"], "", RESET);
        order.manifest = order
            .manifest
            .replace("authority = \"FederalRule\"", "authority = \"CaseOrder\"");
        compile(&[package("rho", &[], "", RHO), order])
    })
}

/// Due dates by calendar arithmetic alone.
struct Plus;

impl Clock for Plus {
    fn due(&self, r: &DueRequest) -> Result<String, String> {
        litigate_sema::types::date::add_days(&r.trigger, r.amount).ok_or_else(|| "bad".into())
    }

    fn nominal(&self, r: &DueRequest) -> Result<String, String> {
        self.due(r)
    }
}

fn run(events: &[&str]) -> Vec<String> {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    let text = format!(
        "case t\nuses rho ^1.0.0, reset ^1.0.0\nas_of 2011-06-01\nsummary \"s\"\n\n\
         parties {{\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n  c: Court Tribunal\n}}\n\
         events {{\n{events}}}\n"
    );
    let packages = packages();
    let checked = check(&text, packages, Some(&lock(packages)), &Plus, Profile::Dev);
    let mut out = outcome(&checked);
    out.extend(
        checked
            .obligations
            .iter()
            .map(|o| format!("obligation {} {:?}", o.deadline, o.status)),
    );
    out
}

#[test]
fn a_delegation_on_motion_takes_effect_only_once_moved() {
    let rows: &[(&str, &[&str])] = &[
        (
            "moved before the order: the order governs, the answer is timely",
            &[
                "2011-01-04 Served(complaint) on d",
                "2011-01-20 ExtensionMoved() by d",
                "2011-02-10 Answered(complaint) by d",
            ],
        ),
        (
            "never moved: the order has no effect (L3004), the answer is late",
            &[
                "2011-01-04 Served(complaint) on d",
                "2011-02-10 Answered(complaint) by d",
            ],
        ),
        (
            "moved after the order: too late to license it",
            &[
                "2011-01-04 Served(complaint) on d",
                "2011-02-05 ExtensionMoved() by d",
                "2011-02-10 Answered(complaint) by d",
            ],
        ),
    ];
    let out: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, events)| (*name, run(events)))
        .collect();
    insta::assert_debug_snapshot!(out);
}

#[test]
fn the_capability_carries_its_condition_and_procedure() {
    let delegation = packages()[0]
        .modules
        .iter()
        .flat_map(|m| &m.declarations)
        .find_map(|d| match d {
            Declaration::Deadline(x) => x.meta.delegations.first().cloned(),
            _ => None,
        })
        .expect("a delegation");
    insta::assert_json_snapshot!(delegation);
}
