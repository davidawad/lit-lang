//! A pack's `filingLimits` to `limit` declarations (SPEC §3.7, §8.1 read
//! in reverse). A value the pack left out is a hole whose reason is the
//! one `lowered.holes` lists at its pointer; an entry cite the pack's
//! citation syntax does not recognize has no spelling (an entry's cite is
//! never a hole), so it is L6002.
use crate::node::Spelling;
use crate::text;
use litigate_engine::model::{FilingLimit, Hole, Measure, Pack};

/// The keyword of `m`.
const fn measure(m: Measure) -> &'static str {
    match m {
        Measure::Words => "words",
        Measure::Pages => "pages",
        Measure::Lines => "lines",
    }
}

/// The hole `pack` lists at `pointer`.
fn listed<'p>(pack: &'p Pack, pointer: &str) -> Option<&'p Hole> {
    pack.lowered
        .iter()
        .flat_map(|l| &l.holes)
        .find(|h| h.pointer == pointer)
}

/// `?`, or `?("reason")` when `hole` gives one.
fn spelled(hole: Option<&Hole>) -> String {
    hole.and_then(|h| h.reason.as_deref())
        .map_or_else(|| "?".to_owned(), |r| format!("?({})", text::string(r)))
}

/// The `?` or `?("reason")` of the hole `pack` lists at `pointer`.
fn hole(pack: &Pack, pointer: &str) -> String {
    spelled(listed(pack, pointer))
}

impl Spelling<'_> {
    /// The ` cite "..."` of an entry, or nothing.
    fn entry_cite(&mut self, cite: Option<&String>, pointer: &str) -> String {
        let Some(raw) = cite else {
            return String::new();
        };
        let spelled = self.cite(raw);
        if spelled.starts_with("cite ?") {
            self.issues.report(
                pointer,
                format!("entry cite `{raw}` has no exact .lit spelling: the pack's citation syntax does not recognize it"),
            );
        }
        format!(" {spelled}")
    }

    /// Every `filingLimits` entry of `pack` as a `limit` declaration.
    pub(crate) fn limits(&mut self, pack: &Pack) -> Vec<String> {
        pack.filing_limits
            .iter()
            .enumerate()
            .map(|(i, l)| self.limit(pack, l, &format!("/filingLimits/{i}")))
            .collect()
    }

    fn limit(&mut self, pack: &Pack, l: &FilingLimit, base: &str) -> String {
        let id = self.issues.ident(&l.id, &format!("{base}/id"), false);
        let mut entries = Vec::new();
        for (j, b) in l.bounds.iter().enumerate() {
            let at = format!("{base}/bounds/{j}");
            let max = b
                .max
                .map_or_else(|| hole(pack, &format!("{at}/max")), |n| n.to_string());
            let cite = self.entry_cite(b.cite.as_ref(), &format!("{at}/cite"));
            entries.push(format!("{} <= {max}{cite}", measure(b.measure)));
        }
        for (j, c) in l.certificates.iter().enumerate() {
            let at = format!("{base}/certificates/{j}");
            let kind = self.issues.ident(&c.kind, &format!("{at}/kind"), false);
            let measures = if c.measures.is_empty() {
                String::new()
            } else {
                let names: Vec<&str> = c.measures.iter().map(|m| measure(*m)).collect();
                format!(" for [{}]", names.join(", "))
            };
            let cite = self.entry_cite(c.cite.as_ref(), &format!("{at}/cite"));
            entries.push(format!("requires certificate {kind}{measures}{cite}"));
        }
        if let Some(x) = &l.excludes {
            let at = format!("{base}/excludes");
            let mut items: Vec<String> = x
                .items
                .iter()
                .enumerate()
                .map(|(k, item)| self.issues.ident(item, &format!("{at}/items/{k}"), false))
                .collect();
            // Parts not yet identified: `open/0`, `open/1`, ... in order.
            items.extend(
                (0..)
                    .map_while(|n| listed(pack, &format!("{at}/open/{n}")))
                    .map(|h| spelled(Some(h))),
            );
            let cite = self.entry_cite(x.cite.as_ref(), &format!("{at}/cite"));
            entries.push(format!("excludes [{}]{cite}", items.join(", ")));
        }
        for (j, e) in l.ecf.iter().enumerate() {
            let at = format!("{base}/ecf/{j}");
            let code = e
                .code
                .as_deref()
                .map_or_else(|| hole(pack, &format!("{at}/code")), text::string);
            let cite = self.entry_cite(e.cite.as_ref(), &format!("{at}/cite"));
            entries.push(format!("ecf {code}{cite}"));
        }
        let cite = l
            .cite
            .as_ref()
            .map(|c| format!(" {}", self.cite(c)))
            .unwrap_or_default();
        let body: Vec<String> = entries.iter().map(|e| format!("  {e}\n")).collect();
        format!("/// imported\nlimit {id} {{\n{}}}{cite}", body.concat())
    }
}
