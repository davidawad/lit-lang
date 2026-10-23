//! Bounded proofs over the interpreter's arithmetic and roll loop.
use super::super::calendar::Weekday;
use super::super::date::Date;
use super::super::rule::{CountingRule, Toward};
use super::Clock;

fn weekend_clock() -> CountingRule {
    CountingRule {
        id: String::new(),
        label: String::new(),
        source_url: String::new(),
        closed_weekdays: vec![Weekday::Saturday, Weekday::Sunday],
        holidays: Vec::new(),
        min_length: None,
        steps: Vec::new(),
        hours: None,
    }
}

#[kani::proof]
#[kani::unwind(4)]
fn roll_forward_terminates_for_bounded_dates() {
    let epoch_day: i32 = kani::any();
    kani::assume(epoch_day >= -730_000 && epoch_day <= 730_000);
    let rule = weekend_clock();
    let clock = Clock {
        rule: &rule,
        calendars: Vec::new(),
    };
    let start = Date::from_epoch_day(i64::from(epoch_day));
    let rolled = clock.roll(start, Toward::Later, &[]);
    assert!(!clock.is_closed(rolled, &[]));
    assert!(rolled >= start);
}

#[kani::proof]
#[kani::unwind(20)]
fn forward_business_counting_is_after_or_equal_to_trigger() {
    let epoch_day: i32 = kani::any();
    let days: u8 = kani::any();
    kani::assume(epoch_day >= -730_000 && epoch_day <= 730_000);
    kani::assume(days <= 14);
    let rule = weekend_clock();
    let clock = Clock {
        rule: &rule,
        calendars: Vec::new(),
    };
    let trigger = Date::from_epoch_day(i64::from(epoch_day));
    let result = clock.advance_business(trigger, i64::from(days), true, &[]);
    assert!(result >= trigger);
}
