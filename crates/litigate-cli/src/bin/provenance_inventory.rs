//! Read-only Rust inventory of provenance wording on `.lit` note lines.
//! Run with `cargo run -p litigate-cli --bin provenance_inventory -- lit`.

use std::{env, fs, path::Path};

const MARKERS: &[(&str, &str)] = &[
    ("PLACEHOLDER", "placeholder"),
    ("UNVERIFIED", "unverified"),
    ("teaching estimate", "teaching estimate"),
    ("APPROXIMATE", "approximate"),
    ("vintage <date>", "vintage "),
    ("basis: estimate", "basis: estimate"),
];
const FIELDS: &[(&str, &str)] = &[
    ("probability", "probability"), ("p", "probability"),
    ("payoff", "payoff"), ("payoff_by_flag", "payoffByFlag"),
    ("cost", "cost"), ("effort", "effort"), ("duration", "duration"),
    ("hours", "effort"), ("stake", "stake"), ("rate", "probability"),
];

#[derive(Clone)]
struct Decl { start: usize, end: usize, header: String }

fn is_decl(s: &str) -> bool {
    ["node ", "state ", "decision ", "terminal ", "edge ", "chance ", "calibration "]
        .iter().any(|p| s.trim_start().starts_with(p))
}

fn declarations(lines: &[&str]) -> Vec<Decl> {
    let starts: Vec<usize> = lines.iter().enumerate().filter_map(|(i, s)| {
        (is_decl(s) || s.contains(" -> ")).then_some(i)
    }).collect();
    starts.iter().enumerate().map(|(n, &start)| {
        let next = starts.get(n + 1).copied().unwrap_or(lines.len());
        // A transition (`... -> ... {`) is a declaration in its own right.
        // Bounding it at the next declaration keeps sibling edge values from
        // being attributed to the current edge's note.
        let end = next.saturating_sub(1);
        Decl { start, end, header: lines[start].trim().to_string() }
    }).collect()
}

fn marker_names(line: &str) -> Vec<&'static str> {
    let lower = line.to_ascii_lowercase();
    MARKERS.iter().filter_map(|(name, needle)| {
        let found = if *name == "APPROXIMATE" { line.contains("APPROXIMATE") }
            else { lower.contains(&needle.to_ascii_lowercase()) };
        found.then_some(*name)
    }).collect()
}

fn field_kind(line: &str) -> Option<(&'static str, &'static str)> {
    let lower = line.trim_start().to_ascii_lowercase();
    if let Some((_, tail)) = lower.split_once("->") {
        let words: Vec<_> = tail.split_whitespace().collect();
        if words.windows(2).any(|pair| pair[0] == "p" && pair[1] != "{") {
            return Some(("p", "probability"));
        }
    }
    FIELDS.iter().find_map(|(prefix, kind)| {
        let valid = lower.strip_prefix(prefix)?;
        (valid.starts_with(' ') || valid.starts_with(':')).then_some((*prefix, *kind))
    })
}

fn walk(path: &Path) {
    let Ok(source) = fs::read_to_string(path) else { return };
    let lines: Vec<_> = source.lines().collect();
    let decls = declarations(&lines);
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim();
        if !line.starts_with("note ") { continue; }
        let markers = marker_names(line);
        if markers.is_empty() { continue; }
        let decl = decls.iter().find(|d| d.start <= i && i <= d.end);
        let fields: Vec<String> = decl.into_iter().flat_map(|d| {
            lines.iter().enumerate().take(d.end + 1).skip(d.start).filter_map(|(j, s)| {
                field_kind(s).map(|(_, kind)| format!("{}:{kind}", j + 1))
            })
        }).collect();
        let note_lower = line.to_ascii_lowercase();
        let stated_kind = if note_lower.contains("placeholder payoff")
                || note_lower.contains("neutral placeholder")
                || (note_lower.contains("placeholder inputs") && note_lower.contains("payoff"))
                || note_lower.contains("unverified dollar figure") { Some("payoff") }
            else if note_lower.contains("basis: estimate") && note_lower.contains("hours:") { Some("effort") }
            else if note_lower.contains("unverified") && note_lower.contains("docketing fee") { Some("cost") }
            else if note_lower.contains("teaching estimate") && note_lower.contains("prior for the fact-tagged") { Some("probability") }
            else if note_lower.contains("vintage ") && note_lower.contains("hours:") { Some("effort") }
            else { None };
        let number_kinds: Vec<_> = fields.iter().filter_map(|f| f.split_once(':').map(|(_, k)| k)).collect();
        let unambiguous = stated_kind.is_some_and(|kind| number_kinds.iter().filter(|k| **k == kind).count() == 1);
        println!("{}:{}\t{}\t{}\t{}\t{}\t{}", path.display(), i + 1,
            if unambiguous { "unambiguous" } else { "ambiguous" }, markers.join(","),
            decl.map_or("<no declaration>", |d| d.header.as_str()),
            if fields.is_empty() { "<no numeric fields>".into() } else { fields.join(",") }, line);
    }
}

fn walk_dir(path: &Path) {
    let Ok(entries) = fs::read_dir(path) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() { walk_dir(&p); }
        else if p.extension().is_some_and(|ext| ext == "lit") { walk(&p); }
    }
}

fn main() {
    let root = env::args().nth(1).unwrap_or_else(|| "lit".into());
    walk_dir(Path::new(&root));
}
