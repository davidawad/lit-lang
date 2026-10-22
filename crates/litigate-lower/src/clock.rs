//! `counting` and `calendar` declarations to the engine's clock library
//! (SPEC §8.2): one `packs/clock`-style fragment holding the package's
//! `CountingRule` and the `HolidayCalendar`s it closes on. `Pack.counting`
//! names the rule by id.
//!
//! The rule's step pipeline is fixed by SPEC §5.3: anchor, count, base,
//! last-day roll, service extension (forward only) and a second roll, then
//! the clerk-inaccessibility extension. Each step's trace text is
//! generated from the declaration's `cite`.
use crate::diag::Sink;
use crate::source_map::{Entry, SourceMap};
use crate::Build;
use litgraph::clock::calendar::{
    Cycle, Effective, Holiday, HolidayDate as EngineDate, Observance as EngineObservance, Weekday,
};
use litgraph::clock::rule::{
    AnchorKind, CountUnit, Hours, MinLength, Period, Roll, ServiceExtension, Step, Toward,
};
use litgraph::clock::{ClockLibrary, CountingRule, Date, HolidayCalendar, ServiceMethod, Unit};
use litigate_base::{DiagnosticCode as C, Span};
use litigate_sema::hir::{self, Anchor, DayUnit, Declaration, HolidayDate, Observance, RollOn};

/// The clock fragment of `pkg` and its source map, when the package
/// declares the counting rule its pack names.
pub(crate) fn fragment(
    build: &Build<'_, '_>,
    pkg: &hir::Package,
    decls: &[&Declaration],
    counting: Option<&str>,
    sink: &mut Sink,
) -> Option<(ClockLibrary, SourceMap)> {
    let c = decls.iter().find_map(|d| match d {
        Declaration::Counting(c) if Some(c.meta.id.as_str()) == counting => Some(c),
        _ => None,
    })?;
    let span = c.meta.span.as_ref();
    let at = span.cloned().unwrap_or_else(|| sink.fallback.clone());
    let calendar = match build
        .selection
        .resolve(&pkg.manifest.pack_id, &c.calendar, at)
    {
        Ok(Declaration::Calendar(cal)) => cal,
        Ok(_) => {
            sink.error(
                C::TypeMismatch,
                span,
                format!("`holidays {}` does not name a calendar", c.calendar),
            );
            return None;
        }
        Err(d) => {
            sink.out.push(d);
            return None;
        }
    };
    let url = source_url(&pkg.manifest, c.meta.cite.as_deref());
    let Some(source_url) = url.clone() else {
        sink.unsupported(
            span,
            format!(
                "counting rule `{}` needs a `cite` bound to a source with a `url` \
                 (it is the engine's `CountingRule.sourceUrl`)",
                c.meta.id
            ),
        );
        return None;
    };
    let rule = rule(sink, c, &calendar.meta.id, source_url)?;
    let cal = holiday_calendar(sink, calendar, &pkg.manifest)?;
    let library = ClockLibrary {
        schema_version: litgraph::clock::library::CLOCK_SCHEMA_VERSION,
        calendars: vec![cal],
        counting_rules: vec![rule],
    };
    let checked = serde_json::to_string(&library)
        .map_err(|e| e.to_string())
        .and_then(|text| ClockLibrary::from_json(&text).map_err(|e| e.to_string()));
    if let Err(why) = checked {
        sink.error_with(
            C::LoweringUnsupported,
            span,
            format!(
                "counting rule `{}` is not a valid engine clock rule",
                c.meta.id
            ),
            why,
        );
        return None;
    }
    let entry = |pointer: &str, id: &str, meta: &hir::Metadata| Entry {
        pointer: pointer.to_owned(),
        id: id.to_owned(),
        span: meta.span.clone(),
        decl: Some(crate::select::Selection::versioned(meta)),
        cite: meta.cite.as_ref().map(|text| crate::source_map::CiteMap {
            text: text.clone(),
            sections: Vec::new(),
        }),
    };
    let map = SourceMap {
        kind: "clock".to_owned(),
        id: c.meta.id.clone(),
        package: Some(build.package_ref(pkg)),
        entries: vec![
            entry("/countingRules/0", &c.meta.id, &c.meta),
            entry("/calendars/0", &calendar.meta.id, &calendar.meta),
        ],
        ..build.map_header()
    };
    Some((library, map))
}

/// The `url` of the source a cite resolves to: the longest matching
/// `cite_prefixes` of a `[sources.*]`, else a `[[cite_syntax.rules]]`
/// prefix bound to a source (SPEC §4.1, §4.3).
fn source_url(m: &hir::Manifest, cite: Option<&str>) -> Option<String> {
    let cite = cite?;
    let by_prefix = m
        .sources
        .iter()
        .flat_map(|s| s.cite_prefixes.iter().map(move |p| (p, s.id.as_str())));
    let by_rule = m.cite_syntax.rules.iter().flat_map(|r| {
        r.prefixes
            .iter()
            .filter_map(move |p| Some((p, r.source.as_deref()?)))
    });
    let (_, source) = by_prefix
        .chain(by_rule)
        .filter(|(p, _)| cite.starts_with(p.as_str()))
        .max_by_key(|(p, _)| p.len())?;
    m.sources
        .iter()
        .find(|s| s.id == source)
        .and_then(|s| s.url.clone())
}

/// The engine service method for a `.lit` method name (`Mail`,
/// `OtherConsented`): its kebab-case spelling in the engine's closed set.
fn service_method(name: &str) -> Option<ServiceMethod> {
    let mut kebab = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() && i > 0 {
            kebab.push('-');
        }
        kebab.push(ch.to_ascii_lowercase());
    }
    serde_json::from_value(serde_json::Value::String(kebab)).ok()
}

/// The count step's periods: the short-period rule first (SPEC §5.3 step
/// 3), then calendar and court days.
fn periods(sink: &mut Sink, c: &hir::Counting, cite: &str, span: Option<&Span>) -> Vec<Period> {
    let mut out = Vec::new();
    if let Some(short) = c.short_period {
        let Ok(below) = i64::try_from(short.threshold) else {
            sink.unsupported(span, "the short-period threshold is out of range");
            return out;
        };
        let count = match short.unit {
            DayUnit::Calendar => CountUnit::Calendar,
            DayUnit::Court | DayUnit::Business => CountUnit::Business,
        };
        out.push(Period {
            unit: Some(Unit::Calendar),
            below: Some(below),
            count,
            text: format!(
                "{cite}: a period shorter than {below} days counts {n} {what} day(s) {{after}} \
                 the anchor -> {{date}}",
                n = "{n}",
                what = if count == CountUnit::Calendar {
                    "calendar"
                } else {
                    "business"
                },
            ),
        });
    }
    out.push(Period {
        unit: Some(Unit::Calendar),
        below: None,
        count: CountUnit::Calendar,
        text: format!("{cite}: count {{n}} calendar day(s) {{after}} the anchor -> {{date}}"),
    });
    out.push(Period {
        unit: Some(Unit::Court),
        below: None,
        count: CountUnit::Business,
        text: format!(
            "{cite}: count {{n}} court day(s) {{forward}}, skipping weekends and holidays -> {{date}}"
        ),
    });
    out
}

/// The anchor step: `trigger_day` with `exclude_trigger_day` (day 0 is the
/// trigger), or `next_business_day` without it (the first business day
/// after the trigger is day 1).
fn anchor(sink: &mut Sink, c: &hir::Counting, cite: &str, span: Option<&Span>) -> Option<Step> {
    match (c.anchor, c.exclude_trigger_day) {
        (Anchor::TriggerDay, true) => Some(Step::Anchor {
            kind: AnchorKind::Trigger,
            text: format!("{cite}: exclude the trigger date {{trigger}}"),
        }),
        (Anchor::NextBusinessDay, false) => Some(Step::Anchor {
            kind: AnchorKind::AdjacentBusinessDay,
            text: format!(
                "{cite}: the first business day {{after}} {{trigger}} is day 1 -> {{date}}"
            ),
        }),
        (Anchor::TriggerDay, false) => {
            sink.unsupported(
                span,
                "a `trigger_day` anchor must `exclude_trigger_day` (the engine counts from the \
                 day after the trigger)",
            );
            None
        }
        (Anchor::NextBusinessDay, true) => {
            sink.unsupported(
                span,
                "a `next_business_day` anchor is day 1 in the engine; drop `exclude_trigger_day`",
            );
            None
        }
    }
}

/// The last-day roll, or `None` when nothing rolls. Weekends and legal
/// holidays roll together: the engine rolls off any closed day.
fn roll(
    sink: &mut Sink,
    c: &hir::Counting,
    cite: &str,
    span: Option<&Span>,
) -> Option<Option<Step>> {
    let weekend = c.roll_on.contains(&RollOn::Weekend);
    let holiday = c.roll_on.contains(&RollOn::LegalHoliday);
    if !c.include_last_day {
        sink.unsupported(
            span,
            "the engine always includes the last day; add `include_last_day`",
        );
        return None;
    }
    if weekend != holiday {
        sink.unsupported(
            span,
            "`roll_forward_on` must list both `weekend` and `legal_holiday`, or neither: the \
             engine rolls off every closed day",
        );
        return None;
    }
    Some(weekend.then(|| {
        let text = |dir: &str| {
            format!("{cite}: the last day is a weekend or legal holiday; roll {dir} -> {{date}}")
        };
        Step::Roll {
            forward: Roll {
                toward: Toward::Later,
                text: text("forward"),
            },
            backward: Roll {
                toward: Toward::Earlier,
                text: text("backward"),
            },
        }
    }))
}

/// The service-extension step (forward periods only).
fn service(
    sink: &mut Sink,
    c: &hir::Counting,
    cite: &str,
    span: Option<&Span>,
) -> Option<Option<Step>> {
    if c.service_extensions.is_empty() {
        return Some(None);
    }
    let mut methods = Vec::new();
    for (name, (days, unit)) in &c.service_extensions {
        let Some(method) = service_method(name) else {
            sink.unsupported(
                span,
                format!("service method `{name}` is not one the engine recognizes"),
            );
            return None;
        };
        let (DayUnit::Calendar, Ok(days)) = (unit, i64::try_from(*days)) else {
            sink.unsupported(
                span,
                format!("service method `{name}` must add calendar days"),
            );
            return None;
        };
        methods.push(ServiceExtension { method, days });
    }
    Some(Some(Step::Service {
        methods,
        text: format!("{cite}: service by {{method}} adds {{days}} day(s) -> {{date}}"),
        backward_text: format!("{cite}: service extensions apply to forward periods only; ignored"),
    }))
}

/// `c` as an engine counting rule closing on calendar `calendar`.
fn rule(
    sink: &mut Sink,
    c: &hir::Counting,
    calendar: &str,
    source_url: String,
) -> Option<CountingRule> {
    let span = c.meta.span.as_ref();
    let cite = c.meta.cite.clone().unwrap_or_else(|| c.meta.id.clone());
    let anchor = anchor(sink, c, &cite, span);
    let periods = periods(sink, c, &cite, span);
    let roll = roll(sink, c, &cite, span);
    let service = service(sink, c, &cite, span);
    let (anchor, roll, service) = (anchor?, roll?, service?);
    let min_length = matches!(
        anchor,
        Step::Anchor {
            kind: AnchorKind::AdjacentBusinessDay,
            ..
        }
    )
    .then(|| MinLength {
        days: 1,
        message: format!("{cite}: a period must be at least 1 day"),
    });
    let clerk = c
        .roll_on
        .contains(&RollOn::Inaccessible)
        .then(|| Step::ClerkInaccessible {
            text: Some(format!(
                "{cite}: the clerk's office is inaccessible on the last day; extend -> {{date}}"
            )),
            ignored_text: format!("{cite}: no inaccessibility extension applies; ignored"),
        });
    let second_roll = service.as_ref().and(roll.clone());
    let steps = [
        Some(anchor),
        Some(Step::Count { periods }),
        Some(Step::Base {}),
        roll,
    ]
    .into_iter()
    .chain([service, second_roll, clerk])
    .flatten()
    .collect();
    Some(CountingRule {
        id: c.meta.id.clone(),
        label: cite.clone(),
        source_url,
        closed_weekdays: vec![Weekday::Saturday, Weekday::Sunday],
        holidays: vec![calendar.to_owned()],
        min_length,
        steps,
        hours: c.hours.then(|| Hours {
            text: format!("{cite}: count every hour from the trigger; +{{n}} hour(s)"),
            roll_text: format!(
                "{cite}: the period ends on a weekend or legal holiday; continue to the same \
                 time on {{date}}"
            ),
        }),
    })
}

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
/// (widened by a day each side when observance may move it).
fn holiday(sink: &mut Sink, h: &hir::Holiday, span: Option<&Span>) -> Option<Holiday> {
    let observed = match h.observed {
        None => EngineObservance::Actual,
        Some(Observance::NearestWeekday) => EngineObservance::NearestWeekday,
        Some(Observance::NextDay) => {
            sink.unsupported(
                span,
                format!(
                    "holiday `{}`: `observed next_day` has no engine observance (only \
                     `nearest_weekday`)",
                    h.id
                ),
            );
            return None;
        }
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
            from = from.max(Date::from_ymd(i32::from(*from_year), 1, 1).ok()?);
            EngineDate::Fixed {
                month: u32::from(*month),
                day: u32::from(*day),
            }
        }
        HolidayDate::Once { date: on } => {
            let on = date(sink, on)?;
            let slack = i64::from(observed == EngineObservance::NearestWeekday);
            from = from.max(on.add_days(-slack));
            let end = on.add_days(1 + slack);
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
fn holiday_calendar(
    sink: &mut Sink,
    cal: &hir::Calendar,
    m: &hir::Manifest,
) -> Option<HolidayCalendar> {
    let span = cal.meta.span.as_ref();
    let holidays: Vec<Option<Holiday>> = cal
        .holidays
        .iter()
        .map(|h| holiday(sink, h, span))
        .collect();
    Some(HolidayCalendar {
        id: cal.meta.id.clone(),
        title: if cal.meta.label.is_empty() {
            cal.meta.id.clone()
        } else {
            cal.meta.label.clone()
        },
        cite: cal.meta.cite.clone(),
        source_url: source_url(m, cal.meta.cite.as_deref()),
        holidays: holidays.into_iter().collect::<Option<_>>()?,
    })
}
