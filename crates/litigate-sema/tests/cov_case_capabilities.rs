//! Characterization and emergency grants in replay (SPEC §5.3, §5.6;
//! design review §2.4): a late act under a claim-processing deadline is
//! an objection the opponent may raise or forfeit (L4010, a warning), and
//! under a jurisdictional one it is untimely with no exception (L4002); an
//! emergency `grant` in force on an order's date adds its capabilities to
//! the deadlines it reaches, lifting a lock only with `unlock`, and only
//! when it is recorded by the matter's `known_at` and its authority
//! reaches the deadline's.
mod cov_case_support;

use cov_case_support::{check, compile, lock, outcome, package};
use litigate_base::Profile;
use litigate_sema::case::{Clock, DueRequest};
use litigate_sema::hir::{Declaration, Package};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

const KAPPA: &str = "module kappa
start open

enum Doc { complaint }

authority FederalRule > CaseOrder

event Served(doc: Doc)
event Answered(doc: Doc)
event Entered()
event Appealed()
event Moved()
event Ordered(until: Date) extends answer to until
event OrderedLocked(until: Date) extends locked to until
event Released() waives answer

counting oc cite \"FRCP 6(a)\" { exclude_trigger_day, include_last_day, holidays ocal }
calendar ocal { new-year: fixed(1, 1) effective [1870-06-28, ..) observed nearest_weekday }

deadline answer: 10.days from Served(complaint) counted_by oc known_at 2009-01-01 {
  bearer Defendant
  characterization claim_processing cite \"FRCP 12(a)\"
}
deadline appeal: 10.days from Entered() counted_by oc known_at 2009-01-01 {
  characterization jurisdictional cite \"FRCP 4\"
}
deadline locked: 10.days from Entered() counted_by oc known_at 2009-01-01 { extend { never cite \"FRCP 6(b)(2)\" } }

grant emergency on [locked, answer] effective [2020-03-15, 2020-06-13) known_at 2020-03-20 cite \"FRCP 87(c)(2)\" {
  unlock
  extend { by Court, discretion GoodCause, to grant_order_entered + 30.days }
  waive { by Court }
}

state open: \"Open\" known_at 2009-01-01
{
  answer \"Answer\" -> open by Defendant { event Answered(complaint), within answer }
  appeal \"Appeal\" -> open by Plaintiff { event Appealed(), within appeal }
  move \"Move\" -> open by Plaintiff { event Moved() }
}
";

fn kappa() -> &'static [Package] {
    static P: OnceLock<Vec<Package>> = OnceLock::new();
    P.get_or_init(|| compile(&[package("kappa", &[], "", KAPPA)]))
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

fn matter(known_at: &str, events: &[&str]) -> String {
    let events = events.iter().fold(String::new(), |mut s, e| {
        let _ = writeln!(s, "  {e}");
        s
    });
    format!(
        "case t\nuses kappa ^1.0.0\nas_of 2025-01-01\n{known_at}summary \"s\"\n\n\
         parties {{\n  p: Plaintiff Corporation\n  d: Defendant Corporation\n  c: Court Tribunal\n}}\n\
         events {{\n{events}}}\n"
    )
}

fn run(packages: &[Package], known_at: &str, events: &[&str]) -> Vec<String> {
    let checked = check(
        &matter(known_at, events),
        packages,
        Some(&lock(packages)),
        &Plus,
        Profile::Dev,
    );
    let mut out = outcome(&checked);
    out.extend(checked.obligations.iter().map(|o| {
        format!(
            "obligation {} {:?}{}",
            o.deadline,
            o.status,
            o.extended_to
                .as_ref()
                .map(|x| format!(" extended {x}"))
                .unwrap_or_default()
        )
    }));
    out
}

/// `kappa` with its grant changed by `edit`.
fn edited(edit: impl Fn(&mut litigate_sema::hir::Grant)) -> Vec<Package> {
    let mut hir = kappa().to_vec();
    for d in hir[0]
        .modules
        .iter_mut()
        .flat_map(|m| m.declarations.iter_mut())
    {
        if let Declaration::Grant(g) = d {
            edit(g);
        }
    }
    hir
}

#[test]
#[allow(clippy::too_many_lines)] // one table row per case
fn character_and_emergency_grants_in_replay() {
    let without_unlock = edited(|g| g.unlock = false);
    let below = edited(|g| g.meta.authority = "CaseOrder".into());
    let rows: &[(&str, &[Package], &str, &[&str])] = &[
        (
            "claim-processing: a late answer is a forfeitable objection (L4010, warning)",
            kappa(),
            "",
            &[
                "2011-01-04 Served(complaint) on d",
                "2011-01-20 Answered(complaint) by d",
            ],
        ),
        (
            "jurisdictional: a late appeal is untimely, no exception (L4002)",
            kappa(),
            "",
            &["2011-01-04 Entered() by c", "2011-01-20 Appealed() by p"],
        ),
        (
            "a grant in force unlocks and extends a locked deadline",
            kappa(),
            "",
            &[
                "2020-04-01 Entered() by c",
                "2020-04-05 OrderedLocked(2020-05-01) by c cite \"FRCP 87(c)(2)\"",
            ],
        ),
        (
            "outside its window the lock holds",
            kappa(),
            "",
            &[
                "2020-07-01 Entered() by c",
                "2020-07-05 OrderedLocked(2020-07-30) by c cite \"FRCP 6(b)\"",
            ],
        ),
        (
            "a grant the matter's known_at has not recorded adds nothing",
            kappa(),
            "known_at 2020-03-18\n",
            &[
                "2020-04-01 Entered() by c",
                "2020-04-05 OrderedLocked(2020-05-01) by c cite \"FRCP 87(c)(2)\"",
            ],
        ),
        (
            "without `unlock`, a grant's extension does not reach a locked deadline",
            &without_unlock,
            "",
            &[
                "2020-04-01 Entered() by c",
                "2020-04-05 OrderedLocked(2020-05-01) by c cite \"FRCP 87(c)(2)\"",
            ],
        ),
        (
            "a grant below the deadline's authority adds nothing",
            &below,
            "",
            &[
                "2020-04-01 Entered() by c",
                "2020-04-05 OrderedLocked(2020-05-01) by c cite \"FRCP 87(c)(2)\"",
            ],
        ),
        (
            "a grant's waiver releases an open answer in its window",
            kappa(),
            "",
            &[
                "2020-04-01 Served(complaint) on d",
                "2020-04-05 Released() by c cite \"FRCP 87(c)(2)\"",
            ],
        ),
        (
            "an extension beyond the grant's bound is refused",
            kappa(),
            "",
            &[
                "2020-04-01 Served(complaint) on d",
                "2020-04-05 Ordered(2020-06-30) by c cite \"FRCP 87(c)(2)\"",
            ],
        ),
    ];
    let out: BTreeMap<&str, Vec<String>> = rows
        .iter()
        .map(|(name, packages, known_at, events)| (*name, run(packages, known_at, events)))
        .collect();
    insta::assert_debug_snapshot!(out);
}
