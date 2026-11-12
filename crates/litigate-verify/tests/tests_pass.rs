//! `tests-pass` from source: each way an inline `test` assertion is
//! evaluated, holds, fails, or cannot be evaluated, one program per path.
//! Each snapshot holds the `tests-pass` finding and its diagnostics.
mod support;

use litigate_base::Profile;
use litigate_verify::{Obligation, ObligationStatus};
use support::{edit, outcome, verify_one, with, BASE};

const PERSONAL: &str = "given Served(complaint, Personal) on 2025-03-03\n  \
                        expect deadline(answer-period) == 2025-03-24";

/// `BASE`'s first test with `PERSONAL` replaced by `body`.
fn first_test(body: &str) -> String {
    edit(PERSONAL, body)
}

/// Snapshots the `tests-pass` outcome of `lit` and checks its status.
macro_rules! tests_pass {
    ($test:ident, $status:ident, $lit:expr) => {
        #[test]
        fn $test() {
            let checked = verify_one(&$lit, Profile::Strict);
            let found = outcome(&checked, Obligation::TestsPass);
            assert_eq!(
                found.finding.map(|f| f.status),
                Some(ObligationStatus::$status),
                "{found:#?}"
            );
            insta::assert_json_snapshot!(found);
        }
    };
}

/// Three services, two on one day; the earliest is 2025-02-03.
const SERVICES: &str = "given Served(complaint, Personal) on 2025-03-03\n  \
                        given Served(complaint, Personal) on 2025-03-03\n  \
                        given Served(complaint, Personal) on 2025-02-03\n  \
                        expect deadline(answer-period) == ";

tests_pass!(
    the_first_matching_given_triggers_a_first_policy_deadline,
    Passed,
    first_test(&format!("{SERVICES}2025-02-24"))
);

#[test]
fn counting_insensitive_lint_fires_on_weekdays_and_clears_on_a_rolled_boundary() {
    let weekday_only = BASE
        .replace(
            "test \"mail service adds three days\" cite \"FRCP 6(d)\"\n{\n  given Served(complaint, Mail) on 2025-03-03\n  expect deadline(answer-period) == 2025-03-27\n}\n",
            "",
        );
    let checked = verify_one(&weekday_only, Profile::Strict);
    let warning = checked
        .verification
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "L9011")
        .expect("strict L9011");
    assert_eq!(warning.severity, litigate_base::Severity::Error);
    let dev = verify_one(&weekday_only, Profile::Dev);
    let warning = dev
        .verification
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "L9011")
        .expect("dev L9011");
    assert_eq!(warning.severity, litigate_base::Severity::Warning);

    let boundary = format!(
        "{weekday_only}\ntest \"answer due rolls Sunday\" cite \"FRCP 6(a)\"\n{{\n  given Served(complaint, Personal) on 2025-03-02\n  expect deadline(answer-period) == 2025-03-24\n}}"
    );
    let checked = verify_one(&boundary, Profile::Strict);
    assert!(!checked
        .verification
        .diagnostics
        .iter()
        .any(|d| d.code.as_str() == "L9011"));
}

#[test]
fn lowering_failure_preserves_counting_insensitive_lint_output() {
    let source = BASE.replace("payoff 0.usd", "payoff 0.eur");
    let checked = verify_one(&source, Profile::Strict);
    assert!(
        checked.lowered.is_none(),
        "the unsupported currency must fail lowering"
    );
    assert!(!checked
        .verification
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "L9011"));
}

#[test]
fn l9011_fix_is_machine_applicable_and_generates_a_passing_counting_test() {
    let source = BASE.replace(
        "test \"mail service adds three days\" cite \"FRCP 6(d)\"\n{\n  given Served(complaint, Mail) on 2025-03-03\n  expect deadline(answer-period) == 2025-03-27\n}\n",
        "",
    );
    let checked = verify_one(&source, Profile::Strict);
    let diagnostic = checked
        .verification
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "L9011")
        .expect("L9011");
    assert_eq!(diagnostic.severity, litigate_base::Severity::Error);
    assert_eq!(
        diagnostic.fixes[0].applicability,
        litigate_base::Applicability::MachineApplicable
    );
    let dev = verify_one(&source, Profile::Dev);
    let diagnostic = dev
        .verification
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "L9011")
        .expect("dev L9011");
    assert_eq!(diagnostic.severity, litigate_base::Severity::Warning);
    let fix = diagnostic.fixes.first().expect("boundary fix");
    assert_eq!(
        fix.applicability,
        litigate_base::Applicability::MachineApplicable
    );
    let replacement = &fix.edits[0].replacement;
    assert!(!replacement.contains('<'));
    assert!(!replacement.contains("naive"));
    let mut applied = source.clone();
    applied.push_str(replacement);
    for profile in [Profile::Dev, Profile::Strict] {
        let checked = verify_one(&applied, profile);
        assert!(
            !checked
                .verification
                .diagnostics
                .iter()
                .any(|d| d.code.as_str() == "L9011"),
            "{profile:?}"
        );
    }
    let checked = verify_one(&applied, Profile::Strict);
    assert_eq!(
        outcome(&checked, Obligation::TestsPass)
            .finding
            .map(|finding| finding.status),
        Some(ObligationStatus::Passed)
    );
    let fixes_again = checked
        .verification
        .diagnostics
        .iter()
        .filter(|d| !d.fixes.is_empty())
        .count();
    assert_eq!(fixes_again, 0);
}

#[test]
fn l9011_without_a_roll_keeps_the_warning_without_a_fix() {
    let source = BASE
        .replace("  roll_forward_on [weekend, legal_holiday]\n", "")
        .replace(
            "test \"mail service adds three days\" cite \"FRCP 6(d)\"\n{\n  given Served(complaint, Mail) on 2025-03-03\n  expect deadline(answer-period) == 2025-03-27\n}\n",
            "",
        );
    let checked = verify_one(&source, Profile::Strict);
    let diagnostic = checked
        .verification
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "L9011")
        .expect("L9011 remains for a non-rolling rule");
    assert!(diagnostic.fixes.is_empty());
    assert_eq!(diagnostic.severity, litigate_base::Severity::Warning);
    assert!(diagnostic
        .notes
        .iter()
        .any(|n| n.message.contains("no discriminating date")));
}

#[test]
fn l9011_generates_fixes_for_forward_calendar_and_court_days() {
    for declaration in [
        "deadline answer-period: 1.days from Served(complaint) counted_by frcp6",
        "deadline answer-period: 1.court_days from Served(complaint) counted_by frcp6",
    ] {
        let source = BASE
            .replace(
                "deadline answer-period: 21.days from Served(complaint) counted_by frcp6",
                declaration,
            )
            .replace(
                "test \"mail service adds three days\" cite \"FRCP 6(d)\"\n{\n  given Served(complaint, Mail) on 2025-03-03\n  expect deadline(answer-period) == 2025-03-27\n}\n",
                "",
            );
        let source = source
            .replace("== 2025-03-24", "== 2025-03-04")
            .replace("== 2025-03-27", "== 2025-03-07");
        let checked = verify_one(&source, Profile::Strict);
        let diagnostic = checked
            .verification
            .diagnostics
            .iter()
            .find(|d| d.code.as_str() == "L9011")
            .unwrap_or_else(|| {
                panic!(
                    "missing L9011 for {declaration}: {:#?}",
                    checked.verification.diagnostics
                )
            });
        let fix = diagnostic.fixes.first().expect("boundary fix");
        assert_eq!(
            fix.applicability,
            litigate_base::Applicability::MachineApplicable
        );
        assert!(fix.edits[0]
            .replacement
            .contains("Counting boundary: answer-period"));
    }
}

#[test]
fn l9011_anchor_starts_when_the_counting_rule_first_applies() {
    let source = BASE
        .replace(
            "counting frcp6 cite \"FRCP 6(a)\"",
            "counting frcp6 effective [2020-01-01, ..) cite \"FRCP 6(a)\"",
        )
        .replace(
            "counted_by frcp6 cite \"FRCP 12(a)\"",
            "counted_by frcp6 effective [2000-01-01, ..) cite \"FRCP 12(a)\"",
        )
        .replace(
            "test \"mail service adds three days\" cite \"FRCP 6(d)\"\n{\n  given Served(complaint, Mail) on 2025-03-03\n  expect deadline(answer-period) == 2025-03-27\n}\n",
            "",
        );
    let checked = verify_one(&source, Profile::Strict);
    let diagnostic = checked
        .verification
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "L9011")
        .expect("L9011 with a delayed counting-rule version");
    let fix = &diagnostic.fixes[0].edits[0].replacement;
    let date = fix
        .split(" on ")
        .nth(1)
        .and_then(|tail| tail.get(..10))
        .expect("generated given date");
    assert!(date >= "2020-01-01", "generated date {date} predates rule");
    let mut applied = source;
    applied.push_str(fix);
    for profile in [Profile::Dev, Profile::Strict] {
        let checked = verify_one(&applied, profile);
        assert!(
            !checked
                .verification
                .diagnostics
                .iter()
                .any(|d| d.code.as_str() == "L9011"),
            "{profile:?}"
        );
    }
    let checked = verify_one(&applied, Profile::Strict);
    assert_eq!(
        outcome(&checked, Obligation::TestsPass)
            .finding
            .map(|finding| finding.status),
        Some(ObligationStatus::Passed)
    );
}

#[test]
fn l9011_generates_fixes_for_backward_calendar_and_court_days() {
    for unit in ["days", "court_days"] {
        let source = BASE
            .replace(
                "rule duty answer-duty cite \"FRCP 12(a)\"\n{\n  deadline answer-period: 21.days from Served(complaint) counted_by frcp6 cite \"FRCP 12(a)\"\n  {\n    extend { by court, discretion GoodCause }\n  }\n}",
                &format!("deadline answer-period: 1.{unit} before Served(complaint) counted_by frcp6 cite \"FRCP 12(a)\""),
            )
            .replace("event Answered(complaint), within answer-period", "event Answered(complaint)")
            .replace("Served(complaint, Personal) on 2025-03-03", "Served(complaint, Personal) on 2025-03-05")
            .replace(
                "== 2025-03-24",
                "== 2025-03-04",
            )
            .replace(
                "test \"mail service adds three days\" cite \"FRCP 6(d)\"\n{\n  given Served(complaint, Mail) on 2025-03-03\n  expect deadline(answer-period) == 2025-03-27\n}\n",
                "",
            );
        let checked = verify_one(&source, Profile::Strict);
        let diagnostic = checked
            .verification
            .diagnostics
            .iter()
            .find(|d| d.code.as_str() == "L9011")
            .unwrap_or_else(|| {
                panic!(
                    "missing backward L9011 for {unit}: {:#?}",
                    checked.verification.diagnostics
                )
            });
        let fix = &diagnostic.fixes[0].edits[0].replacement;
        let mut applied = source;
        applied.push_str(fix);
        let checked = verify_one(&applied, Profile::Strict);
        assert!(
            !checked
                .verification
                .diagnostics
                .iter()
                .any(|d| d.code.as_str() == "L9011"),
            "{unit}"
        );
        assert_eq!(
            outcome(&checked, Obligation::TestsPass)
                .finding
                .map(|finding| finding.status),
            Some(ObligationStatus::Passed),
            "{unit}"
        );
    }
}

tests_pass!(
    the_latest_matching_given_triggers_a_latest_policy_deadline,
    Passed,
    first_test(&format!("{SERVICES}2025-03-24")).replacen(
        "counted_by frcp6 cite \"FRCP 12(a)\"\n  {\n    extend { by court, discretion GoodCause }\n  }",
        "counted_by frcp6 cite \"FRCP 12(a)\" { trigger latest, extend { by court, discretion GoodCause } }",
        1
    )
);

tests_pass!(
    date_order_assertions_hold_and_fail,
    Failed,
    first_test(
        "given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(answer-period) <= 2025-03-24\n  \
         expect deadline(answer-period) > 2025-03-20\n  \
         expect deadline(answer-period) != 2025-03-24"
    )
);

tests_pass!(
    court_days_count_on_the_engine_clock,
    Passed,
    edit("21.days from", "21.court_days from")
        .replacen("== 2025-03-24", "== 2025-04-01", 1)
        .replacen("== 2025-03-27", "== 2025-04-04", 1)
);

tests_pass!(
    a_failed_has_event_shows_the_call,
    Failed,
    first_test(
        "given Served(complaint, Personal) on 2025-03-03\n  \
         expect has_event(Answered(complaint)) == true\n  \
         expect deadline(answer-period) == 2025-03-24"
    )
);

tests_pass!(
    a_failed_literal_comparison_shows_both_values,
    Failed,
    first_test(
        "given Served(complaint, Personal) on 2025-03-03\n  \
         expect 1 == 2\n  \
         expect deadline(answer-period) == 2025-03-24"
    )
);

tests_pass!(
    no_deadline_version_in_force_on_the_trigger,
    Failed,
    first_test(
        "given Served(complaint, Personal) on 1980-03-03\n  \
         expect deadline(answer-period) == 1980-03-24"
    )
    .replacen(
        "counted_by frcp6 cite \"FRCP 12(a)\"",
        "counted_by frcp6 effective [1990-01-01, ..) cite \"FRCP 12(a)\"",
        1
    )
);

/// The answer period's counting rule `frcp6` split at 2000: before, an
/// earlier version given by `old` (its items); a test of a 1995 trigger.
fn split_counting(old: &str, version: &str) -> String {
    with(
        "test \"answer period in 1995\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 1995-03-03\n  \
         expect deadline(answer-period) == 1995-03-24\n}",
    )
    .replacen(
        "counting frcp6 cite \"FRCP 6(a)\"\n",
        &format!(
            "counting frcp6 effective [1990-01-01, 2000-01-01){version} cite \"FRCP 6(a)\"\n\
             {{\n{old}\n}}\n\ncounting frcp6 effective [2000-01-01, ..) cite \"FRCP 6(a)\"\n"
        ),
        1,
    )
}

/// Personal delivery adds nothing; the fixtures' rules recognize no other
/// service method.
const OLD_RULES: &str = "  exclude_trigger_day\n  include_last_day\n  \
                         roll_forward_on [weekend, legal_holiday]\n  holidays federal\n  \
                         anchor trigger_day\n  service_extension { Personal: 0.days, \
                         Electronic: unrecognized, Mail: unrecognized, Clerk: unrecognized, \
                         OtherConsented: unrecognized, CommercialCarrier: unrecognized, \
                         PrisonMail: unrecognized, MailForeign: unrecognized, \
                         Express: unrecognized, ExpressForeign: unrecognized }";

tests_pass!(
    each_trigger_counts_under_the_counting_version_in_force,
    Passed,
    split_counting(OLD_RULES, "")
);

tests_pass!(
    no_counting_version_in_force_on_the_trigger,
    Failed,
    edit(
        "counting frcp6 cite \"FRCP 6(a)\"",
        "counting frcp6 effective [2000-01-01, ..) cite \"FRCP 6(a)\""
    )
    .replacen(
        PERSONAL,
        "given Served(complaint, Personal) on 1995-03-03\n  \
         expect deadline(answer-period) == 1995-03-24",
        1
    )
);

tests_pass!(
    a_counting_version_that_does_not_lower_fails_its_tests,
    Failed,
    split_counting(
        "  include_last_day\n  holidays federal\n  anchor trigger_day",
        ""
    )
);

tests_pass!(
    a_counting_version_not_yet_known_is_not_selected,
    Failed,
    split_counting(OLD_RULES, " known_at 2030-01-01")
);

tests_pass!(
    the_engine_clock_rejects_a_period_it_cannot_count,
    Failed,
    with(
        "counting nbd cite \"FRCP 6(a)\"\n{\n  anchor next_business_day\n  include_last_day\n  \
         holidays federal\n  service_extension { Personal: 0.days, Electronic: unrecognized, Mail: unrecognized, Clerk: unrecognized, OtherConsented: unrecognized, CommercialCarrier: unrecognized, PrisonMail: unrecognized, MailForeign: unrecognized, Express: unrecognized, ExpressForeign: unrecognized }\n}\n\
         rule duty zero-duty cite \"FRCP 12(a)\"\n{\n  deadline zero-period: 0.days from \
         Served(complaint) counted_by nbd\n}\n\
         test \"a zero-day period\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(zero-period) == 2025-03-03\n}"
    )
);

tests_pass!(
    weeks_months_and_years_count_with_nominal_and_adjusted_dates,
    Passed,
    with(
        "counting monthly cite \"FRCP 6(a)\"\n{\n  exclude_trigger_day\n  include_last_day\n  \
         roll_forward_on [weekend, legal_holiday]\n  holidays federal\n  end_of_month clamp\n  service_extension { Personal: 0.days, Electronic: unrecognized, Mail: unrecognized, Clerk: unrecognized, OtherConsented: unrecognized, CommercialCarrier: unrecognized, PrisonMail: unrecognized, MailForeign: unrecognized, Express: unrecognized, ExpressForeign: unrecognized }\n}\n\
         rule duty period-duty cite \"FRCP 12(a)\"\n{\n  \
         deadline week-period: 2.weeks from Served(complaint) counted_by monthly\n  \
         deadline month-period: 3.months from Served(complaint) counted_by monthly\n  \
         deadline year-period: 1.years from Served(complaint) counted_by monthly\n}\n\
         test \"calendar periods\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 2024-11-30\n  \
         expect deadline(week-period) == 2024-12-16\n  \
         expect deadline(month-period).nominal == 2025-02-28\n  \
         expect deadline(month-period).adjusted == 2025-02-28\n  \
         expect deadline(year-period).nominal == 2025-11-30\n  \
         expect deadline(year-period) == 2025-12-01\n}"
    )
);

// `quick` displaces `answer-period` (SPEC §5.5): its own test holds, and
// the base tests expecting 21 days fail.
tests_pass!(
    a_displacing_claimant_counts_the_deadline,
    Failed,
    with(
        "deadline quick: 14.days from Served(complaint) counted_by frcp6 \
         notwithstanding answer-period\n\
         test \"the claimant counts\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(answer-period) == 2025-03-17\n}"
    )
);

tests_pass!(
    a_displacing_claimant_counts_from_its_own_trigger,
    Failed,
    with(
        "deadline moved: 7.days from Answered(complaint) counted_by frcp6 \
         notwithstanding answer-period\n\
         test \"the claimant's trigger\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(answer-period) == 2025-03-10\n}"
    )
);

/// A deadline whose due date the trigger designates (`designated due`).
const DESIGNATED: &str = "event Noticed(due: Date)\n\
deadline noticed-answer: 30.days from Noticed() counted_by frcp6 { designated due }\n";

tests_pass!(
    a_designated_due_date_is_the_given_date_rolled,
    Passed,
    with(&format!(
        "{DESIGNATED}test \"the notice sets the date\" cite \"FRCP 12(a)\"\n{{\n  \
         given Noticed(2025-04-05) on 2025-03-03\n  \
         expect deadline(noticed-answer).nominal == 2025-04-05\n  \
         expect deadline(noticed-answer) == 2025-04-07\n}}"
    ))
);

tests_pass!(
    a_designated_due_date_below_the_period_is_refused,
    Failed,
    with(&format!(
        "{DESIGNATED}test \"too short a notice\" cite \"FRCP 12(a)\"\n{{\n  \
         given Noticed(2025-03-20) on 2025-03-03\n  \
         expect deadline(noticed-answer) == 2025-03-20\n}}"
    ))
);

tests_pass!(
    a_chained_period_counts_from_the_end_it_names,
    Passed,
    with(
        "rule duty chain-duty cite \"FRCP 12(a)\"\n{\n  \
         deadline after-answer: 10.days from deadline(answer-period).nominal counted_by frcp6\n  \
         deadline after-after: 10.days from deadline(after-answer).adjusted counted_by frcp6\n}\n\
         test \"chained periods\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(after-answer) == 2025-04-03\n  \
         expect deadline(after-after) == 2025-04-14\n}"
    )
);

tests_pass!(
    a_cycle_of_chained_periods_is_not_counted,
    Failed,
    with(
        "rule duty loop-duty cite \"FRCP 12(a)\"\n{\n  \
         deadline loop-a: 1.days from deadline(loop-b).nominal counted_by frcp6\n  \
         deadline loop-b: 1.days from deadline(loop-a).nominal counted_by frcp6\n}\n\
         test \"a cycle\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(loop-a) == 2025-03-04\n}"
    )
);

tests_pass!(
    a_cycle_through_the_version_in_force_is_not_counted,
    Failed,
    with(
        "rule duty loop-duty cite \"FRCP 12(a)\"\n{\n  \
         deadline loop-a: 1.days from Served(complaint) counted_by frcp6 effective [2009-12-01, 2020-01-01)\n  \
         deadline loop-a: 1.days from deadline(loop-b).nominal counted_by frcp6 effective [2020-01-01, ..)\n  \
         deadline loop-b: 1.days from deadline(loop-a).nominal counted_by frcp6\n}\n\
         test \"a cycle in force\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(loop-a) == 2025-03-04\n}"
    )
);

tests_pass!(
    a_chain_on_a_period_the_clock_cannot_count_is_not_counted,
    Failed,
    with(
        "counting nbd cite \"FRCP 6(a)\"\n{\n  anchor next_business_day\n  include_last_day\n  \
         holidays federal\n}\n\
         rule duty zero-duty cite \"FRCP 12(a)\"\n{\n  deadline zero-period: 0.days from \
         Served(complaint) counted_by nbd\n  \
         deadline after-zero: 1.days from deadline(zero-period).adjusted counted_by frcp6\n}\n\
         test \"after a zero-day period\" cite \"FRCP 12(a)\"\n{\n  \
         given Served(complaint, Personal) on 2025-03-03\n  \
         expect deadline(after-zero) == 2025-03-04\n}"
    )
);
