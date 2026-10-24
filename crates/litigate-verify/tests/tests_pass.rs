//! `tests-pass` from source: each way an inline `test` assertion is
//! evaluated, holds, fails, or cannot be evaluated, one program per path.
//! Each snapshot holds the `tests-pass` finding and its diagnostics.
mod support;

use litigate_base::Profile;
use litigate_verify::{Obligation, ObligationStatus};
use support::{edit, outcome, verify_one, with};

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

tests_pass!(
    the_latest_matching_given_triggers_a_latest_policy_deadline,
    Passed,
    first_test(&format!("{SERVICES}2025-03-24")).replacen(
        "counted_by frcp6\n",
        "counted_by frcp6 { trigger latest }\n",
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
        "counted_by frcp6\n",
        "counted_by frcp6 effective [1990-01-01, ..)\n",
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

const OLD_RULES: &str = "  exclude_trigger_day\n  include_last_day\n  \
                         roll_forward_on [weekend, legal_holiday]\n  holidays federal\n  \
                         anchor trigger_day";

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
         holidays federal\n}\n\
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
         roll_forward_on [weekend, legal_holiday]\n  holidays federal\n  end_of_month clamp\n}\n\
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
