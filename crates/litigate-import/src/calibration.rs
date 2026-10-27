//! A calibration set to a `calibration` declaration in the package it
//! calibrates (`lit/<dir>/calibration-<id>.lit`): every entry's ref names
//! that one pack (`pack::local`, spelled `local` inside it) or an instance
//! of the links package (`base@origin::local`, kept qualified), and each
//! distinct source becomes one `source` the entries cite.
use crate::issues::Issues;
use crate::text;
use crate::Emitted;
use litigate_engine::api::{
    CalibrationEntry, CalibrationSet, CalibrationSource, CalibrationTarget,
};
use litigate_engine::clock::Date;

/// Imports `set`; `dir_of` maps a pack id to its package directory.
pub(crate) fn import_calibration(
    set: &CalibrationSet,
    from: &str,
    dir_of: &dyn Fn(&str) -> String,
    issues: &mut Issues,
) -> Vec<Emitted> {
    let id = issues.ident(&set.id, "/id", false);
    let mut owner: Option<String> = None;
    // Each distinct source with the pointer of the first entry citing it.
    let mut sources: Vec<(&CalibrationSource, String)> = Vec::new();
    let mut entries = Vec::new();
    for (index, entry) in set.entries.iter().enumerate() {
        let pointer = format!("/entries/{index}");
        let (package, target) = target(&entry.target_ref, &pointer, issues, dir_of);
        match &owner {
            Some(dir) if *dir != package => issues.report(
                &format!("{pointer}/ref"),
                format!(
                    "a calibration set calibrates one package: `{}` is not in `{dir}`",
                    entry.target_ref
                ),
            ),
            Some(_) => {}
            None => owner = Some(package),
        }
        let source = if let Some(i) = sources.iter().position(|(s, _)| **s == entry.source) {
            i + 1
        } else {
            sources.push((&entry.source, format!("{pointer}/source")));
            sources.len()
        };
        let value = value(entry, &target, &pointer, issues);
        let mut opts = vec![format!("source source-{source}"), format!("n {}", entry.n)];
        if let Some(method) = &entry.method {
            opts.push(format!("method {}", text::string(method)));
        }
        entries.push(format!("  {value} {{\n    {}\n  }}", opts.join("\n    ")));
    }
    let dir = owner.unwrap_or_else(|| {
        issues.report(
            "/entries",
            "a calibration set with no entries names no package",
        );
        String::new()
    });
    let mut body = Vec::new();
    if let Some(description) = &set.description {
        body.push(format!("  note {}", text::string(description)));
    }
    for (index, (source, pointer)) in sources.iter().enumerate() {
        body.push(source_text(index + 1, source, pointer, issues));
    }
    body.extend(entries);
    let text = [
        format!("// Imported by `litigate import` from {from}."),
        format!("module {dir}.calibration-{id}"),
        format!(
            "calibration {id}: {} {{\n{}\n}}",
            text::string(&set.title),
            body.join("\n")
        ),
    ]
    .join("\n\n");
    vec![Emitted::lit(
        format!("lit/{dir}/calibration-{id}.lit"),
        text,
    )]
}

/// The value part of an entry: `p T q`, `duration T D` or `attr T name x`.
fn value(entry: &CalibrationEntry, target: &str, pointer: &str, issues: &mut Issues) -> String {
    match entry.target {
        CalibrationTarget::EdgeProbability => {
            let p = match entry.value {
                Some(v) => issues.spell(text::number(v), &format!("{pointer}/value"), "value"),
                None => "?".to_owned(),
            };
            format!("p {target} {p}")
        }
        CalibrationTarget::EdgeDuration => {
            let d = entry.distribution.as_ref().map_or_else(String::new, |d| {
                let mut days = |v: f64, key: &str| {
                    issues.spell(
                        text::duration(v, "days", false),
                        &format!("{pointer}/distribution/{key}"),
                        "duration",
                    )
                };
                if d.min.is_none() && d.max.is_none() {
                    return days(d.mode, "mode");
                }
                let mut parts = Vec::new();
                if let Some(min) = d.min {
                    parts.push(format!("min {}", days(min, "min")));
                }
                parts.push(format!("mode {}", days(d.mode, "mode")));
                if let Some(max) = d.max {
                    parts.push(format!("max {}", days(max, "max")));
                }
                format!("{{ {} }}", parts.join(", "))
            });
            format!("duration {target} {d}")
        }
        CalibrationTarget::NodeAttr => {
            let attr = entry.attr.as_deref().unwrap_or_default();
            let attr = issues.ident(attr, &format!("{pointer}/attr"), false);
            let v = entry.value.unwrap_or_default();
            let v = issues.spell(text::number(v), &format!("{pointer}/value"), "value");
            format!("attr {target} {attr} {v}")
        }
    }
}

/// `(package directory, spelling)` of an entry's ref.
fn target(
    r: &str,
    pointer: &str,
    issues: &mut Issues,
    dir_of: &dyn Fn(&str) -> String,
) -> (String, String) {
    let pointer = format!("{pointer}/ref");
    let Some((qualifier, local)) = r.split_once("::") else {
        issues.report(&pointer, format!("`{r}` names no pack (`pack::local`)"));
        return (String::new(), String::new());
    };
    issues.ident(r, &pointer, true);
    if qualifier.contains('@') {
        ("links".to_owned(), r.to_owned())
    } else {
        (dir_of(qualifier), local.to_owned())
    }
}

/// `source source-N "Title" { url, vintage [start, end + 1), retrieved }`.
fn source_text(index: usize, s: &CalibrationSource, pointer: &str, issues: &mut Issues) -> String {
    let date = |text: &str, key: &str, issues: &mut Issues| {
        Date::parse_iso(text)
            .ok()
            .filter(|d| d.to_iso() == text)
            .or_else(|| {
                issues.report(
                    &format!("{pointer}/{key}"),
                    format!("`{text}` is not a date `YYYY-MM-DD`"),
                );
                None
            })
    };
    let start = date(&s.vintage_start, "vintageStart", issues).map(Date::to_iso);
    let end = match &s.vintage_end {
        Some(end) => date(end, "vintageEnd", issues).map(|d| d.add_days(1).to_iso()),
        None => Some("..".to_owned()),
    };
    let mut fields = vec![
        format!("url {}", text::string(&s.url)),
        format!(
            "vintage [{}, {})",
            start.unwrap_or_default(),
            end.unwrap_or_default()
        ),
    ];
    if let Some(retrieved) = &s.retrieved {
        if let Some(d) = date(retrieved, "retrieved", issues) {
            fields.push(format!("retrieved {}", d.to_iso()));
        }
    }
    format!(
        "  source source-{index} {} {{\n    {}\n  }}",
        text::string(&s.title),
        fields.join("\n    ")
    )
}
