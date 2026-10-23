// SPDX-License-Identifier: GPL-3.0-or-later
//! The generic counting-rule interpreter: runs a [`CountingRule`]'s step
//! pipeline over a working date, against the union of its holiday
//! calendars, and emits the rule's own cited trace text. Every
//! forum-specific fact (anchor, short-period unit, roll direction,
//! service-method table, clerk extension, hours provision, the words of
//! each step) is data on the rule.

use std::collections::BTreeSet;

use super::calendar::HolidayCalendar;
use super::date::Date;
use super::rule::{
    AnchorKind, CountUnit, CountingRule, EndOfMonth, Period, Roll, ServiceExtension, Step, Toward,
};
use super::types::{
    ClockConfig, ComputedDeadline, ComputedHourDeadline, DayDeadline, Direction, HourDeadline, Unit,
};
use crate::error::{Error, Result};

#[cfg(kani)]
mod kani_proofs;

/// A counting rule resolved against its holiday calendars: everything the
/// interpreter needs.
#[derive(Debug, Clone)]
pub struct Clock<'a> {
    /// The rule.
    pub rule: &'a CountingRule,
    /// The calendars named by `rule.holidays`, in that order.
    pub calendars: Vec<&'a HolidayCalendar>,
}

impl Clock<'_> {
    /// Every observed holiday in `year` across the rule's calendars.
    #[must_use]
    pub fn holidays_in(&self, year: i32) -> BTreeSet<Date> {
        self.calendars
            .iter()
            .flat_map(|c| c.observed_in(year))
            .collect()
    }

    /// True if `date` is a legal holiday under this rule: a calendar
    /// holiday (checking the neighboring years too, since an observed date
    /// can cross a year boundary, e.g. Jan 1 observed Dec 31) or one of
    /// `additional`.
    #[must_use]
    pub fn is_legal_holiday(&self, date: Date, additional: &[Date]) -> bool {
        let (year, _, _) = date.ymd();
        additional.contains(&date)
            || [year - 1, year, year + 1]
                .into_iter()
                .any(|y| self.holidays_in(y).contains(&date))
    }

    /// True if the forum is closed on `date` (a closed weekday or a legal
    /// holiday).
    #[must_use]
    pub fn is_closed(&self, date: Date, additional: &[Date]) -> bool {
        let wd = date.weekday();
        self.rule.closed_weekdays.iter().any(|w| w.number() == wd)
            || self.is_legal_holiday(date, additional)
    }

    fn roll(&self, mut d: Date, toward: Toward, add: &[Date]) -> Date {
        let step = match toward {
            Toward::Later => 1,
            Toward::Earlier => -1,
        };
        while self.is_closed(d, add) {
            d = d.add_days(step);
        }
        d
    }

    /// Advance `k` business days from `start` (exclusive of `start`).
    fn advance_business(&self, start: Date, k: i64, forward: bool, add: &[Date]) -> Date {
        let step = if forward { 1 } else { -1 };
        let mut d = start;
        let mut counted = 0;
        while counted < k {
            d = d.add_days(step);
            if !self.is_closed(d, add) {
                counted += 1;
            }
        }
        d
    }
}

/// Fill `{placeholder}`s in `t` from `vars`. Templates are validated at
/// load, so an unknown name (impossible for a loaded rule) is left as is.
fn fill(t: &str, vars: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(t.len() + 16);
    let mut rest = t;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            out.push_str(&rest[open..]);
            return out;
        };
        match vars.iter().find(|(k, _)| *k == &after[..close]) {
            Some((_, v)) => out.push_str(v),
            None => out.push_str(&rest[open..=open + close + 1]),
        }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

/// Which optional effects a run applied (reported on the result).
#[derive(Default)]
struct Applied {
    rolled: bool,
    service: bool,
    clerk: bool,
}

/// Working state of one pipeline run.
struct Run<'c, 'a> {
    clock: &'c Clock<'a>,
    dl: &'c DayDeadline,
    cfg: &'c ClockConfig,
    forward: bool,
    anchor: Date,
    current: Date,
    nominal: Date,
    base: Date,
    applied: Applied,
    steps: Vec<String>,
}

impl Run<'_, '_> {
    fn say(&mut self, t: &str, date: Date, extra: &[(&str, &str)]) {
        let (trigger, anchor, date) = (
            self.dl.trigger.to_iso(),
            self.anchor.to_iso(),
            date.to_iso(),
        );
        let n = self.dl.days.to_string();
        let mut vars: Vec<(&str, &str)> = vec![
            ("trigger", &trigger),
            ("anchor", &anchor),
            ("date", &date),
            ("n", &n),
            ("after", if self.forward { "after" } else { "before" }),
            ("forward", if self.forward { "forward" } else { "backward" }),
        ];
        vars.extend_from_slice(extra);
        self.steps.push(fill(t, &vars));
    }

    fn step(&mut self, s: &Step) -> Result<()> {
        match s {
            Step::Anchor { kind, text } => self.anchor(*kind, text),
            Step::Count { periods } => self.count(periods)?,
            Step::Service {
                methods,
                text,
                backward_text,
            } => self.service(methods, text, backward_text)?,
            Step::Base {} => self.base = self.current,
            Step::Roll { forward, backward } => {
                self.roll(if self.forward { forward } else { backward });
            }
            Step::ClerkInaccessible { text, ignored_text } => {
                self.clerk(text.as_deref(), ignored_text);
            }
        }
        Ok(())
    }

    fn anchor(&mut self, kind: AnchorKind, text: &str) {
        let add = &self.cfg.additional_holidays;
        self.anchor = match (kind, self.forward) {
            (AnchorKind::Trigger, _) => self.dl.trigger,
            (AnchorKind::AdjacentBusinessDay, true) => {
                self.clock
                    .roll(self.dl.trigger.add_days(1), Toward::Later, add)
            }
            (AnchorKind::AdjacentBusinessDay, false) => {
                self.clock
                    .roll(self.dl.trigger.add_days(-1), Toward::Earlier, add)
            }
        };
        self.current = self.anchor;
        self.say(text, self.anchor, &[]);
    }

    fn count(&mut self, periods: &[Period]) -> Result<()> {
        if matches!(self.dl.unit, Unit::Month | Unit::Year) {
            return self.count_months();
        }
        let n = self.dl.days;
        let p = periods
            .iter()
            .find(|p| p.unit.is_none_or(|u| u == self.dl.unit) && p.below.is_none_or(|b| n < b))
            .ok_or_else(|| {
                Error::Invalid(format!(
                    "{}: no counting period applies to a {n}-day {:?} deadline",
                    self.clock.rule.label, self.dl.unit
                ))
            })?;
        // An inclusive anchor is day 1 itself.
        let k = match self.clock.rule.anchor_kind() {
            Some(AnchorKind::AdjacentBusinessDay) => n - 1,
            _ => n,
        };
        self.current = match p.count {
            CountUnit::Calendar => self.anchor.add_days(if self.forward { k } else { -k }),
            CountUnit::Business => self.clock.advance_business(
                self.anchor,
                k,
                self.forward,
                &self.cfg.additional_holidays,
            ),
        };
        self.nominal = self.current;
        self.say(&p.text, self.current, &[]);
        Ok(())
    }

    /// Calendar months (a year is twelve) from the anchor to the
    /// corresponding day, under the rule's `months` provision.
    fn count_months(&mut self) -> Result<()> {
        let rule = self.clock.rule;
        let m = rule.months.as_ref().ok_or_else(|| {
            Error::Invalid(format!(
                "{} has no calendar-month provision: month and year periods are not counted",
                rule.label
            ))
        })?;
        if rule.anchor_kind() != Some(AnchorKind::Trigger) {
            return Err(Error::Invalid(format!(
                "{}: a month or year period counts from the trigger date, not an adjacent \
                 business day",
                rule.label
            )));
        }
        let (n, year) = (self.dl.days, self.dl.unit == Unit::Year);
        let months = n
            .checked_mul(if year { 12 } else { 1 })
            .ok_or_else(|| Error::Invalid(format!("a {n}-year period is out of range")))?;
        let (date, short) = self
            .anchor
            .add_months(if self.forward { months } else { -months })?;
        self.current = match (short, m.end_of_month) {
            (true, EndOfMonth::Roll) => date.last_of_month().add_days(1),
            _ => date,
        };
        self.nominal = self.current;
        let unit = match (year, n == 1) {
            (true, true) => "year",
            (true, false) => "years",
            (false, true) => "month",
            (false, false) => "months",
        };
        self.say(&m.text, self.current, &[("unit", unit)]);
        Ok(())
    }

    fn service(
        &mut self,
        methods: &[ServiceExtension],
        text: &str,
        backward_text: &str,
    ) -> Result<()> {
        let Some(method) = self.dl.service_method else {
            return Ok(());
        };
        if !self.forward {
            self.say(backward_text, self.current, &[]);
            return Ok(());
        }
        let ext = methods.iter().find(|m| m.method == method).ok_or_else(|| {
            let valid: Vec<&str> = methods.iter().map(|m| m.method.as_str()).collect();
            Error::Invalid(format!(
                "service method {method:?} is not valid under {}; valid methods: {}",
                self.clock.rule.label,
                valid.join(", ")
            ))
        })?;
        self.current = self.current.add_days(ext.days);
        self.applied.service = true;
        let (m, d) = (format!("{method:?}"), ext.days.to_string());
        self.say(text, self.current, &[("method", &m), ("days", &d)]);
        Ok(())
    }

    fn roll(&mut self, r: &Roll) {
        let next = self
            .clock
            .roll(self.current, r.toward, &self.cfg.additional_holidays);
        if next != self.current {
            self.current = next;
            self.applied.rolled = true;
            self.say(&r.text, next, &[]);
        }
    }

    fn clerk(&mut self, text: Option<&str>, ignored_text: &str) {
        if !self.cfg.clerk_inaccessible {
            return;
        }
        match text {
            Some(t) => {
                let next = self.clock.roll(
                    self.current.add_days(1),
                    Toward::Later,
                    &self.cfg.additional_holidays,
                );
                self.current = next;
                self.applied.clerk = true;
                self.say(t, next, &[]);
            }
            _ => self.say(ignored_text, self.current, &[]),
        }
    }
}

/// Compute a day-counted due date by running `clock`'s pipeline.
///
/// # Errors
/// `Error::Invalid` for a period below the rule's `minLength` (with the
/// rule's own message), a negative period, a deadline no `count` period
/// matches, or a service method the rule does not recognize.
pub fn compute_due_date(
    dl: &DayDeadline,
    clock: &Clock,
    cfg: &ClockConfig,
) -> Result<ComputedDeadline> {
    if let Some(min) = &clock.rule.min_length {
        if dl.days < min.days {
            return Err(Error::Invalid(min.message.clone()));
        }
    }
    if dl.days < 0 {
        return Err(Error::Invalid(format!(
            "days must be non-negative, got {}",
            dl.days
        )));
    }
    // A week is seven calendar days.
    let weeks = (dl.unit == Unit::Week).then(|| DayDeadline {
        days: dl.days.saturating_mul(7),
        unit: Unit::Calendar,
        ..dl.clone()
    });
    let dl = weeks.as_ref().unwrap_or(dl);
    let mut run = Run {
        clock,
        dl,
        cfg,
        forward: matches!(dl.direction, Direction::Forward),
        anchor: dl.trigger,
        current: dl.trigger,
        nominal: dl.trigger,
        base: dl.trigger,
        applied: Applied::default(),
        steps: vec![],
    };
    for s in &clock.rule.steps {
        run.step(s)?;
    }
    Ok(ComputedDeadline {
        rule_set: clock.rule.id.clone(),
        anchor_date: run.anchor.to_iso(),
        nominal_date: run.nominal.to_iso(),
        base_date: run.base.to_iso(),
        due_date: run.current.to_iso(),
        last_day_rolled: run.applied.rolled,
        service_days_added: run.applied.service,
        clerk_inaccessibility_applied: run.applied.clerk,
        steps: run.steps,
    })
}

/// Compute a due instant for an hours-based period under `clock`'s hours
/// provision: count every hour from the trigger instant, then continue to
/// the same time on the next open day if the end falls on a closed day.
///
/// # Errors
/// `Error::Invalid` if the rule has no hours provision or `hours` is
/// negative.
pub fn compute_hour_deadline(
    dl: &HourDeadline,
    clock: &Clock,
    cfg: &ClockConfig,
) -> Result<ComputedHourDeadline> {
    let Some(h) = &clock.rule.hours else {
        return Err(Error::Invalid(format!(
            "{} has no hours-based period provision in the fetched text",
            clock.rule.label
        )));
    };
    if dl.hours < 0 {
        return Err(Error::Invalid(format!(
            "hours must be non-negative, got {}",
            dl.hours
        )));
    }
    let due_instant = dl.trigger_epoch_seconds + dl.hours * 3600;
    let due_day = Date::from_epoch_day(due_instant.div_euclid(86400));
    let n = dl.hours.to_string();
    let mut steps = vec![fill(&h.text, &[("n", &n)])];
    let add = &cfg.additional_holidays;
    let mut final_day = clock.roll(due_day, Toward::Later, add);
    let last_day_rolled = final_day != due_day;
    if last_day_rolled {
        let date = final_day.to_iso();
        steps.push(fill(&h.roll_text, &[("n", &n), ("date", &date)]));
    }
    let inaccessible = h
        .inaccessible_text
        .as_deref()
        .filter(|_| cfg.clerk_inaccessible);
    if let Some(t) = inaccessible {
        final_day = clock.roll(final_day.add_days(1), Toward::Later, add);
        let date = final_day.to_iso();
        steps.push(fill(t, &[("n", &n), ("date", &date)]));
    }
    let seconds_in_day = due_instant.rem_euclid(86400);
    let (hh, mm, ss) = (
        seconds_in_day / 3600,
        (seconds_in_day / 60) % 60,
        seconds_in_day % 60,
    );
    Ok(ComputedHourDeadline {
        due_date: final_day.to_iso(),
        due_datetime: format!("{final_day}T{hh:02}:{mm:02}:{ss:02}"),
        last_day_rolled,
        clerk_inaccessibility_applied: inaccessible.is_some(),
        steps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_substitutes_known_names_and_keeps_the_rest() {
        assert_eq!(
            fill("a {x} b {y} c", &[("x", "1"), ("y", "2")]),
            "a 1 b 2 c"
        );
        assert_eq!(fill("{z} {x}", &[("x", "1")]), "{z} 1");
        assert_eq!(fill("open {x", &[("x", "1")]), "open {x");
        assert_eq!(fill("no braces", &[]), "no braces");
    }
}
