//! Property: any prefix of a valid event history is valid (SPEC §5.6).
//! Replay judges each event only by the events before it, the versions
//! governing its date (which depend on the matter's first event, never on
//! later ones), and deadlines counted from earlier triggers; so cutting a
//! valid history short can't make it invalid.
mod case_support;

use case_support::{check_2009, matter_2009, render};
use proptest::prelude::*;

/// Days since 1970-01-01 to an ISO date (proleptic Gregorian).
fn iso(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// 2009-06-01, six months before the amendments took effect.
const BASE: i64 = 14_396;

#[derive(Debug, Clone)]
struct History {
    served: i64,
    mail: bool,
    prior_rules: Option<i64>,
    extended: Option<i64>,
    respond: i64,
    answer_first: bool,
    denied: i64,
    answer_after: i64,
}

fn history() -> impl Strategy<Value = History> {
    (
        (0i64..365, any::<bool>(), proptest::option::of(0i64..20)),
        (proptest::option::of(0i64..40), 0i64..30, any::<bool>()),
        (1i64..60, 0i64..20),
    )
        .prop_map(
            |(
                (served, mail, prior_rules),
                (extended, respond, answer_first),
                (denied, answer_after),
            )| {
                History {
                    served: BASE + served,
                    mail,
                    prior_rules,
                    extended,
                    respond,
                    answer_first,
                    denied,
                    answer_after,
                }
            },
        )
}

/// The history's event lines, sorted by date (ties in generation order).
fn events(h: &History) -> Vec<String> {
    let method = if h.mail { "Mail" } else { "Personal" };
    let mut out = vec![(h.served, format!("Served(complaint, {method}) on widget"))];
    if let Some(o) = h.prior_rules {
        out.push((h.served + o, "OrderedPriorRules() by court".to_owned()));
    }
    if let Some(e) = h.extended {
        out.push((
            h.served + e,
            "Extended(complaint) by court cite \"FRCP 6(b)(1)(A)\"".to_owned(),
        ));
    }
    let respond = h.served + h.respond;
    if h.answer_first {
        out.push((respond, "Answered(complaint) by widget".to_owned()));
    } else {
        out.push((respond, "MovedToDismiss(complaint) by widget".to_owned()));
        out.push((respond + h.denied, "Denied(motion) by court".to_owned()));
        out.push((
            respond + h.denied + h.answer_after,
            "Answered(complaint) by widget".to_owned(),
        ));
    }
    out.sort_by_key(|(d, _)| *d);
    out.into_iter()
        .map(|(d, line)| format!("{} {line}", iso(d)))
        .collect()
}

#[test]
fn iso_dates_are_civil() {
    assert_eq!(iso(0), "1970-01-01");
    assert_eq!(iso(BASE), "2009-06-01");
    assert_eq!(iso(BASE + 183), "2009-12-01");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn every_prefix_of_a_valid_history_is_valid(h in history()) {
        let lines = events(&h);
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let full = check_2009(&matter_2009(&refs));
        if full.ok() {
            for n in 0..refs.len() {
                let prefix = check_2009(&matter_2009(&refs[..n]));
                prop_assert!(
                    prefix.ok(),
                    "prefix {n} of {refs:#?} fails: {:#?}",
                    render(&prefix.diagnostics)
                );
            }
        }
    }
}
