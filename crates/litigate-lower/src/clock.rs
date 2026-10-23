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
use crate::source_map::{Entry, SourceMap};
use crate::Build;
use litgraph::clock::calendar::Weekday;
use litgraph::clock::rule::{
    AnchorKind, CountUnit, Hours, MinLength, Period, Roll, ServiceExtension, Step, Toward,
};
use litgraph::clock::{ClockLibrary, CountingRule, ServiceMethod, Unit};
use litigate_base::{DiagnosticCode as C, Span};
use litigate_sema::hir::{self, Anchor, DayUnit, Declaration, RollOn};

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
        schema_version: litgraph::clock::library::CLOCK_SCHEMA_VERSION,
        calendars: Vec::new(),
        counting_rules: Vec::new(),
    };
    let mut entries = Vec::new();
    for c in &rules {
        let (rule, calendar) = lowered(build, pkg, c, sink)?;
        entries.push(entry(
            &format!("/countingRules/{}", library.counting_rules.len()),
            &c.meta,
        ));
        library.counting_rules.push(rule);
        if library.calendars.iter().all(|k| k.id != calendar.meta.id) {
            entries.push(entry(
                &format!("/calendars/{}", library.calendars.len()),
                &calendar.meta,
            ));
            library
                .calendars
                .push(calendar::holiday_calendar(sink, calendar, &pkg.manifest)?);
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

/// The source-map entry of one generated rule or calendar.
fn entry(pointer: &str, meta: &hir::Metadata) -> Entry {
    Entry {
        pointer: pointer.to_owned(),
        id: meta.id.clone(),
        span: meta.span.clone(),
        decl: Some(crate::select::Selection::versioned(meta)),
        cite: meta.cite.as_ref().map(|text| crate::source_map::CiteMap {
            text: text.clone(),
            sections: Vec::new(),
        }),
    }
}

/// Counting rule `c` as an engine rule, with the calendar declaration it
/// names (in force at the build date).
fn lowered<'p>(
    build: &Build<'_, 'p>,
    pkg: &hir::Package,
    c: &hir::Counting,
    sink: &mut Sink,
) -> Option<(CountingRule, &'p hir::Calendar)> {
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
    let Some(source_url) = source_url(&pkg.manifest, c.meta.cite.as_deref()) else {
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
    Some((rule(sink, c, &calendar.meta.id, source_url)?, calendar))
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

/// The service-extension step (forward periods only).
fn service(
    sink: &mut Sink,
    c: &hir::Counting,
    cite: &str,
    span: Option<&Span>,
) -> Result<Option<Step>, ()> {
    if c.service_extensions.is_empty() {
        return Ok(None);
    }
    let mut methods = Vec::new();
    for (name, (days, unit)) in &c.service_extensions {
        let Some(method) = service_method(name) else {
            sink.unsupported(
                span,
                format!("service method `{name}` is not one the engine recognizes"),
            );
            return Err(());
        };
        let (DayUnit::Calendar, Ok(days)) = (unit, i64::try_from(*days)) else {
            sink.unsupported(
                span,
                format!("service method `{name}` must add calendar days"),
            );
            return Err(());
        };
        methods.push(ServiceExtension { method, days });
    }
    Ok(Some(Step::Service {
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
    let second_roll = service.as_ref().and(roll.clone());
    // `base` is the day the period "would otherwise expire" (FRCP 6(d),
    // FRAP 26(c)): after its own last-day roll, before any added days.
    let steps = [
        Some(anchor),
        Some(Step::Count { periods }),
        roll,
        Some(Step::Base {}),
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
