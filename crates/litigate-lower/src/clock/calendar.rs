//! Calendar declarations as engine holiday calendars (SPEC §8.2).
use crate::diag::Sink;
use litigate_base::DiagnosticCode as C;
use litigate_engine::clock::calendar::{
    Cycle, Effective, Holiday, HolidayDate as EngineDate, Observance as EngineObservance, Weekday,
};
use litigate_engine::clock::{Date, HolidayCalendar};
use litigate_sema::hir::{self, HolidayDate, Observance};

fn date(sink: &mut Sink, text: &str) -> Option<Date> {
    let parsed = Date::parse_iso(text).ok();
    if parsed.is_none() {
        sink.error(
            C::InvalidDate,
            None,
            format!("`{text}` is not a calendar date"),
        );
    }
    parsed
}

fn weekday(sink: &mut Sink, name: &str) -> Option<Weekday> {
    let parsed = serde_json::from_value(serde_json::Value::String(name.to_owned())).ok();
    if parsed.is_none() {
        sink.error(
            C::InvalidCounting,
            None,
            format!("`{name}` is not a weekday"),
        );
    }
    parsed
}

/// One holiday: date rule, cycle, observance and effective window. `last`
/// is the -1st weekday; `every` is a fixed date on a cycle, effective from
/// its first year; `once` is a fixed date effective for that day only
/// (widened by a day each side when observance may move it: back under
/// `nearest_weekday`, forward under both).
fn holiday(sink: &mut Sink, h: &hir::Holiday) -> Option<Holiday> {
    let observed = match h.observed {
        None => EngineObservance::Actual,
        Some(Observance::NearestWeekday) => EngineObservance::NearestWeekday,
        Some(Observance::NextDay) => EngineObservance::NextDay,
    };
    let mut from = date(sink, &h.in_force.from)?;
    let mut until = match &h.in_force.to {
        Some(to) => Some(date(sink, to)?),
        None => None,
    };
    let mut cycle = None;
    let rule = match &h.date {
        HolidayDate::Fixed { month, day } => EngineDate::Fixed {
            month: u32::from(*month),
            day: u32::from(*day),
        },
        HolidayDate::Nth {
            n,
            weekday: w,
            month,
        } => EngineDate::NthWeekday {
            month: u32::from(*month),
            weekday: weekday(sink, w)?,
            nth: i32::from(*n),
        },
        HolidayDate::Last { weekday: w, month } => EngineDate::NthWeekday {
            month: u32::from(*month),
            weekday: weekday(sink, w)?,
            nth: -1,
        },
        HolidayDate::Every {
            years,
            from_year,
            month,
            day,
        } => {
            cycle = Some(Cycle {
                every: i32::from(*years),
                anchor_year: i32::from(*from_year),
            });
            // January 1 exists in every year, so the fallback never applies.
            from = Date::from_ymd(i32::from(*from_year), 1, 1).map_or(from, |jan1| from.max(jan1));
            EngineDate::Fixed {
                month: u32::from(*month),
                day: u32::from(*day),
            }
        }
        HolidayDate::Once { date: on } => {
            let on = date(sink, on)?;
            let back = i64::from(observed == EngineObservance::NearestWeekday);
            let forward = i64::from(observed != EngineObservance::Actual);
            from = from.max(on.add_days(-back));
            let end = on.add_days(1 + forward);
            until = Some(until.map_or(end, |u| u.min(end)));
            let (_, month, day) = on.ymd();
            EngineDate::Fixed { month, day }
        }
    };
    Some(Holiday {
        id: h.id.clone(),
        name: h.id.clone(),
        date: rule,
        cycle,
        observed,
        effective: Some(Effective {
            from: Some(from),
            until,
        }),
        cite: None,
    })
}

/// A calendar declaration as an engine holiday calendar.
pub(super) fn holiday_calendar(
    sink: &mut Sink,
    cal: &hir::Calendar,
    m: &hir::Manifest,
) -> Option<HolidayCalendar> {
    let holidays: Vec<Option<Holiday>> = cal.holidays.iter().map(|h| holiday(sink, h)).collect();
    Some(HolidayCalendar {
        id: cal.meta.id.clone(),
        title: if cal.meta.label.is_empty() {
            cal.meta.id.clone()
        } else {
            cal.meta.label.clone()
        },
        cite: cal.meta.cite.clone(),
        source_url: super::source_url(m, cal.meta.cite.as_deref()),
        holidays: holidays.into_iter().collect::<Option<_>>()?,
    })
}
