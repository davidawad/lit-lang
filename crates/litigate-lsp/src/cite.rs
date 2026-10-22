//! A `cite` to its L0 text. The sections come from the lowering's
//! source-map sidecar (SPEC §8.6) when the cite belongs to a lowered node
//! or edge, else from the same resolver the lowering uses for it; each
//! section is then narrowed to the cite's pinpoint (`12(a)(1)(A)(i)`) in
//! the vendored corpus text.
use litgraph::cite::{parse_cite_string, CiteGrammar, SourceCorpus};
use litgraph::model::CiteSyntax;
use litigate_cli::verdict::Verdict;
use litigate_lower::source_map::{Citations, L0Section};
use litigate_lower::Document;
use litigate_sema::hir;

/// One cited section located in L0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    /// Corpus heading (`Rule 12`).
    pub heading: String,
    /// The `[sources.*]` binding it resolves through, when bound.
    pub source: Option<String>,
    /// Pinpoint markers (`["a", "1", "A", "i"]`).
    pub pinpoint: Vec<String>,
    /// Repository-relative corpus file, when vendored.
    pub path: Option<String>,
    /// One-based first and last line of the cited text in `path`.
    pub lines: Option<(u32, u32)>,
    /// The cited text.
    pub text: Option<String>,
}

/// The sidecar sections recorded for `cite` in a package's lowering,
/// preferring the entry whose span covers `line`.
fn from_sidecar(verdict: &Verdict, cite: &str, line: u32) -> Option<Vec<L0Section>> {
    let lowered = verdict.lowered.as_ref()?;
    let maps = std::iter::once(&lowered.source_map).chain(lowered.clock.as_ref().map(|(_, m)| m));
    let entries: Vec<_> = maps
        .flat_map(|m| &m.entries)
        .filter(|e| e.cite.as_ref().is_some_and(|c| c.text == cite))
        .collect();
    let covering = entries.iter().find(|e| {
        e.span
            .as_ref()
            .is_some_and(|s| s.line <= line && line <= s.end_line)
    });
    covering
        .or(entries.first())
        .and_then(|e| e.cite.as_ref())
        .map(|c| c.sections.clone())
}

/// The citation syntax `pkg` lowered with.
fn syntax(verdict: Option<&Verdict>) -> CiteSyntax {
    verdict
        .and_then(|v| v.lowered.as_ref())
        .and_then(|l| match &l.document {
            Document::Pack(p) => Some(p.cite_syntax.clone()),
            Document::Links(_) => None,
        })
        .unwrap_or_default()
}

/// Locates `cite`, written on one-based `line` of a module of `pkg`.
#[must_use]
pub fn locate(
    pkg: &hir::Package,
    verdict: Option<&Verdict>,
    cite: &str,
    line: u32,
    corpus: &SourceCorpus,
) -> Vec<Located> {
    let citations = Citations::new(syntax(verdict), &pkg.manifest.sources, Some(corpus));
    let sections = verdict
        .and_then(|v| from_sidecar(v, cite, line))
        .or_else(|| citations.map(Some(&cite.to_owned())).map(|m| m.sections))
        .unwrap_or_default();
    let refs = parse_cite_string(cite, &CiteGrammar::new(citations.syntax()));
    sections
        .into_iter()
        .map(|s| {
            let pinpoint = refs
                .iter()
                .find(|r| r.heading().as_deref() == Some(s.heading.as_str()))
                .map(|r| r.subsections.clone())
                .unwrap_or_default();
            let text = s.path.as_ref().and_then(|p| corpus.files.get(p));
            let found = text
                .zip(s.line)
                .and_then(|(text, line)| narrow(text, line, &pinpoint));
            Located {
                heading: s.heading,
                source: s.source,
                pinpoint,
                path: s.path,
                lines: found.as_ref().map(|(a, b, _)| (*a, *b)),
                text: found.map(|(_, _, t)| t),
            }
        })
        .collect()
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn is_marker(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('(')
        && t[1..].split_once(')').is_some_and(|(m, _)| {
            !m.is_empty() && m.len() <= 5 && m.chars().all(char::is_alphanumeric)
        })
}

/// The lines of the section whose `## ` heading is on one-based `heading`
/// line, narrowed through each pinpoint marker in turn: `(first, last,
/// text)`, one-based and inclusive. A marker the text lacks stops the
/// narrowing at the enclosing division.
#[must_use]
pub fn narrow(text: &str, heading: u32, pinpoint: &[String]) -> Option<(u32, u32, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let start = usize::try_from(heading).ok()?.checked_sub(1)?;
    lines.get(start)?;
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.starts_with("## "))
        .map_or(lines.len(), |i| start + 1 + i);
    let (mut first, mut last) = (start, end);
    for marker in pinpoint {
        let open = format!("({marker})");
        let Some(at) = (first + 1..last).find(|&i| lines[i].trim_start().starts_with(&open)) else {
            break;
        };
        let depth = indent(lines[at]);
        last = (at + 1..last)
            .find(|&i| is_marker(lines[i]) && indent(lines[i]) <= depth + 1)
            .unwrap_or(last);
        first = at;
    }
    let body = lines[first..last].join("\n");
    let one = |i: usize| u32::try_from(i + 1).ok();
    Some((one(first)?, one(last - 1)?, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "## Rule 12\nRule 12 title\n  (a) TIME.\n      (1) In General.\n           (A) A defendant must:\n               (i) within 21 days; or\n               (ii) within 60 days.\n           (B) A party.\n  (b) HOW.\n## Rule 13\n";

    #[test]
    fn narrows_to_the_pinpoint() {
        let pin = ["a", "1", "A", "i"].map(String::from);
        assert_eq!(
            narrow(TEXT, 1, &pin),
            Some((6, 6, "               (i) within 21 days; or".to_owned()))
        );
        let (first, last, _) = narrow(TEXT, 1, &pin[..1]).unwrap_or_default();
        assert_eq!((first, last), (3, 8));
    }

    #[test]
    fn whole_section_without_a_pinpoint_or_a_missing_marker() {
        assert_eq!(narrow(TEXT, 1, &[]).map(|(a, b, _)| (a, b)), Some((1, 9)));
        let missing = ["z".to_owned()];
        assert_eq!(
            narrow(TEXT, 1, &missing).map(|(a, b, _)| (a, b)),
            Some((1, 9))
        );
        assert_eq!(narrow(TEXT, 99, &[]), None);
    }
}
