//! Bounded proofs for date arithmetic and the day-count interpreter.
//! Run with `just verify-kani`.

use super::date::Date;

#[kani::proof]
fn bounded_date_addition_round_trips_without_overflow() {
    let epoch_day: i32 = kani::any();
    let delta: i16 = kani::any();
    kani::assume((-730_000..=730_000).contains(&epoch_day));
    kani::assume((-366..=366).contains(&delta));
    let date = Date::from_epoch_day(i64::from(epoch_day));
    let shifted = date.add_days(i64::from(delta));
    assert_eq!(shifted.epoch_day(), i64::from(epoch_day) + i64::from(delta));
}
