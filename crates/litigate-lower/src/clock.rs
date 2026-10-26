//! `counting` and `calendar` declarations to the engine's clock library
//! (SPEC §8.2): one `packs/clock`-style fragment holding the package's
//! `CountingRule`s and the `HolidayCalendar`s they close on. `Pack.counting`
//! names the rule by id (none in a counting-rule library such as
//! `lit/core`, which declares several).
//!
//! The rule's step pipeline is fixed by SPEC §5.3: anchor, count,
//! last-day roll, base, service extension (forward only) and a second roll,
//! then the clerk-inaccessibility extension. Each step's trace text is
//! generated from the declaration's `cite`.
mod calendar;

use crate::diag::Sink;
use crate::source_map::{Citations, Entry, SourceMap};
use crate::Build;
use litigate_base::{DiagnosticCode as C, Span};
use litigate_engine::clock::calendar::Weekday;
use litigate_engine::clock::rule::{
    AnchorKind, CountUnit, EndOfMonth, Hours, MinLength, Months, Period, Roll, ServiceExtension,
    Step, Toward,
};
use litigate_engine::clock::{ClockLibrary, CountingRule, ServiceMethod, Unit};
use litigate_sema::hir::{self, Anchor, DayUnit, Declaration, RollOn, ServiceTime};

/// The clock fragment of `pkg` and its source map: counting rule `only`
/// when given, else every counting rule the package declares (one, or a
/// counting-rule library's several), each with the calendar it closes on.
/// `None` when the package declares no selected rule.
pub(crate) fn fragment(
    build: &Build<'_, '_>,
    pkg: &hir::Package,
    decls: &[&Declaration],
    only: Option<&str>,
    sink: &mut Sink,
) -> Option<(ClockLibrary, SourceMap)> {
    let rules: Vec<&hir::Counting> = decls
        .iter()
        .filter_map(|d| match d {
            Declaration::Counting(c) if only.is_none_or(|id| id == c.meta.id) => Some(c),
            _ => None,
        })
        .collect();
    let first = rules.first()?;
    let mut library = ClockLibrary {
        schema_version: litigate_engine::clock::library::CLOCK_SCHEMA_VERSION,
        calendars: Vec::new(),
        counting_rules: Vec::new(),
        lowered: Some(build.lowered(Vec::new())),
    };
    let mut entries = Vec::new();
    for c in &rules {
        let (rule, calendars) = lowered(build, pkg, c, sink)?;
        entries.push(entry(
            build,
            pkg,
            &format!("/countingRules/{}", library.counting_rules.len()),
            &c.meta,
        ));
        library.counting_rules.push(rule);
        for calendar in calendars {
            if library.calendars.iter().all(|k| k.id != calendar.meta.id) {
                // A calendar's cite binds through its own package's sources,
                // so every fragment carrying it carries the same calendar.
                let owner = declaring(build, calendar).unwrap_or(pkg);
                entries.push(entry(
                    build,
                    owner,
                    &format!("/calendars/{}", library.calendars.len()),
                    &calendar.meta,
                ));
                library.calendars.push(calendar::holiday_calendar(
                    sink,
                    calendar,
                    &owner.manifest,
                )?);
            }
        }
    }
    // Serializing plain data cannot fail; were it to, the empty text would
    // be rejected below like any other invalid fragment.
    let text = serde_json::to_string(&library).unwrap_or_default();
    if let Err(why) = ClockLibrary::from_json(&text) {
        sink.error_with(
            C::LoweringUnsupported,
            first.meta.span.as_ref(),
            format!(
                "counting rule `{}` is not a valid engine clock rule",
                first.meta.id
            ),
            why.to_string(),
        );
        return None;
    }
    let map = SourceMap {
        kind: "clock".to_owned(),
        id: first.meta.id.clone(),
        package: Some(build.package_ref(pkg)),
        entries,
        ..build.map_header()
    };
    Some((library, map))
}

/// The loaded package declaring `calendar`.
fn declaring<'p>(build: &Build<'_, 'p>, calendar: &hir::Calendar) -> Option<&'p hir::Package> {
    build.packages.iter().copied().find(|p| {
        p.modules
            .iter()
            .flat_map(|m| &m.declarations)
            .any(|d| matches!(d, Declaration::Calendar(c) if std::ptr::eq(c, calendar)))
    })
}

/// The source-map entry of one generated rule or calendar declared in
/// `pkg`, its cite located through `pkg`'s sources.
fn entry(build: &Build<'_, '_>, pkg: &hir::Package, pointer: &str, meta: &hir::Metadata) -> Entry {
    let citations = Citations::of_package(pkg, build.corpus);
    Entry {
        pointer: pointer.to_owned(),
        id: meta.id.clone(),
        span: meta.span.clone(),
        decl: Some(crate::select::Selection::versioned(meta)),
        cite: citations.map_at(meta.cite.as_ref(), Some(&meta.effective.from)),
    }
}

/// The calendars counting rule `c` closes on (in force at the build date):
/// `holidays`, then `declared_holidays` (both closing either direction),
/// then `forward_holidays`.
struct Closing<'p> {
    both: Vec<&'p hir::Calendar>,
    forward: Option<&'p hir::Calendar>,
}

/// The calendar declaration `id` names, in force at the build date.
fn resolve_calendar<'p>(
    build: &Build<'_, 'p>,
    pkg: &hir::Package,
    id: &str,
    at: &Span,
    sink: &mut Sink,
) -> Option<&'p hir::Calendar> {
    match build
        .selection
        .resolve(&pkg.manifest.pack_id, id, at.clone())
    {
        Ok(Declaration::Calendar(cal)) => Some(cal),
        Ok(_) => {
            sink.error(
                C::TypeMismatch,
                Some(at),
                format!("`holidays {id}` does not name a calendar"),
            );
            None
        }
        Err(d) => {
            sink.out.push(d);
            None
        }
    }
}

/// Counting rule `c` as an engine rule, with the calendar declarations it
/// closes on (in force at the build date).
fn lowered<'p>(
    build: &Build<'_, 'p>,
    pkg: &hir::Package,
    c: &hir::Counting,
    sink: &mut Sink,
) -> Option<(CountingRule, Vec<&'p hir::Calendar>)> {
    let span = c.meta.span.as_ref();
    let at = span.cloned().unwrap_or_else(|| sink.fallback.clone());
    let mut both = vec![resolve_calendar(build, pkg, &c.calendar, &at, sink)?];
    if let Some(id) = &c.declared_calendar {
        both.push(resolve_calendar(build, pkg, id, &at, sink)?);
    }
    let forward = match &c.forward_calendar {
        Some(id) => Some(resolve_calendar(build, pkg, id, &at, sink)?),
        None => None,
    };
    let cited = c.meta.cite.as_deref().and_then(|cite| {
        let url = source_url(&pkg.manifest, Some(cite))?;
        Some((cite, url))
    });
    let Some((cite, source_url)) = cited else {
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
    let closing = Closing { both, forward };
    let rule = rule(sink, c, cite, &closing, source_url)?;
    let calendars = closing.both.into_iter().chain(closing.forward).collect();
    Some((rule, calendars))
}

/// The `url` of the source a cite resolves to: the longest matching
/// `cite_prefixes` of a `[sources.*]`, else a `[[cite_syntax.rules]]`
/// prefix bound to a source (SPEC §4.1, §4.3).
pub(crate) fn source_url(m: &hir::Manifest, cite: Option<&str>) -> Option<String> {
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
        .filter(|(p, _)| has_prefix(cite, p))
        .max_by_key(|(p, _)| p.len())?;
    m.sources
        .iter()
        .find(|s| s.id == source)
        .and_then(|s| s.url.clone())
}

/// `cite` starts with `prefix` at a token boundary: `FRCP 6` is a prefix
/// of `FRCP 6(a)` but not of `FRCP 60`.
fn has_prefix(cite: &str, prefix: &str) -> bool {
    let word = |c: char| c.is_alphanumeric();
    cite.strip_prefix(prefix).is_some_and(|rest| {
        !(prefix.chars().next_back().is_some_and(word) && rest.chars().next().is_some_and(word))
    })
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
        // Sema admits only day units here.
        let count = if short.unit == DayUnit::Calendar {
            CountUnit::Calendar
        } else {
            CountUnit::Business
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
) -> Result<Option<Step>, ()> {
    let weekend = c.roll_on.contains(&RollOn::Weekend);
    let holiday = c.roll_on.contains(&RollOn::LegalHoliday);
    if !c.include_last_day {
        sink.unsupported(
            span,
            "the engine always includes the last day; add `include_last_day`",
        );
        return Err(());
    }
    if weekend != holiday {
        sink.unsupported(
            span,
            "`roll_forward_on` must list both `weekend` and `legal_holiday`, or neither: the \
             engine rolls off every closed day",
        );
        return Err(());
    }
    Ok(weekend.then(|| {
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

/// The service-extension step (forward periods only), total over the
/// engine's service methods as sema's `service_extension` is over the
/// language's.
fn service(
    sink: &mut Sink,
    c: &hir::Counting,
    cite: &str,
    span: Option<&Span>,
) -> Result<Step, ()> {
    let mut methods = Vec::new();
    let mut unrecognized = Vec::new();
    for (name, time) in &c.service_extensions {
        let Some(method) = service_method(name) else {
            sink.unsupported(
                span,
                format!("service method `{name}` is not one the engine recognizes"),
            );
            return Err(());
        };
        match time {
            ServiceTime::Unrecognized => unrecognized.push(method),
            ServiceTime::Adds(days, unit) => {
                let (DayUnit::Calendar, Ok(days)) = (unit, i64::try_from(*days)) else {
                    sink.unsupported(
                        span,
                        format!("service method `{name}` must add calendar days"),
                    );
                    return Err(());
                };
                methods.push(ServiceExtension { method, days });
            }
        }
    }
    methods.sort_by_key(|m| m.method);
    unrecognized.sort();
    Ok(Step::Service {
        methods,
        unrecognized,
        text: format!("{cite}: service by {{method}} adds {{days}} day(s) -> {{date}}"),
        backward_text: format!("{cite}: service extensions apply to forward periods only; ignored"),
    })
}

/// `c`, cited as `cite`, as an engine counting rule closing on the
/// `closing` calendars.
fn rule(
    sink: &mut Sink,
    c: &hir::Counting,
    cite: &str,
    closing: &Closing<'_>,
    source_url: String,
) -> Option<CountingRule> {
    let span = c.meta.span.as_ref();
    let anchor = anchor(sink, c, cite, span);
    let periods = periods(sink, c, cite, span);
    let roll = roll(sink, c, cite, span);
    let service = service(sink, c, cite, span);
    let (anchor, roll, service) = (anchor?, roll.ok()?, service.ok()?);
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
    let second_roll = roll.clone();
    // `base` is the day the period "would otherwise expire" (FRCP 6(d),
    // FRAP 26(c)): after its own last-day roll, before any added days.
    let steps = [
        Some(anchor),
        Some(Step::Count { periods }),
        roll,
        Some(Step::Base {}),
    ]
    .into_iter()
    .chain([Some(service), second_roll, clerk])
    .flatten()
    .collect();
    Some(CountingRule {
        id: c.meta.id.clone(),
        label: cite.to_owned(),
        source_url,
        closed_weekdays: vec![Weekday::Saturday, Weekday::Sunday],
        holidays: closing.both.iter().map(|k| k.meta.id.clone()).collect(),
        forward_holidays: closing.forward.iter().map(|k| k.meta.id.clone()).collect(),
        min_length,
        steps,
        hours: c.hours.then(|| Hours {
            text: format!("{cite}: count every hour from the trigger; +{{n}} hour(s)"),
            roll_text: format!(
                "{cite}: the period ends on a weekend or legal holiday; continue to the same \
                 time on {{date}}"
            ),
            inaccessible_text: c.roll_on.contains(&RollOn::Inaccessible).then(|| {
                format!(
                    "{cite}: the clerk's office is inaccessible during the last hour; extend to \
                     the same time on {{date}}"
                )
            }),
        }),
        months: c.end_of_month.map(|policy| Months {
            end_of_month: match policy {
                hir::EndOfMonth::Clamp => EndOfMonth::Clamp,
                hir::EndOfMonth::Roll => EndOfMonth::Roll,
            },
            text: format!(
                "{cite}: count {{n}} {{unit}} {{after}} the anchor to the corresponding day{} -> \
                 {{date}}",
                match policy {
                    hir::EndOfMonth::Clamp => ", else the last day of that month",
                    hir::EndOfMonth::Roll => ", else the first day of the next month",
                }
            ),
        }),
    })
}
