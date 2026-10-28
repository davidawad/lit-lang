//! Filing requirements (SPEC §3.7): a paper kind's length bounds, the
//! certificates it must carry, what does not count toward its length, and
//! the ECF events it is filed under.
use super::{Authored, Metadata};
use litigate_base::Span;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// `limit k { ... }`: one version of paper kind `k`'s filing requirements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Limit {
    /// Identity (the paper kind), versions, authority, relations and the
    /// declaration's cite, which covers every entry without its own.
    pub meta: Metadata,
    /// Length bounds, at most one per measure. Several are alternatives:
    /// a paper is within the limit when it is within any one of them
    /// (FRAP 32(a)(7)(A): 30 pages "unless it complies with" (B)).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounds: Vec<LengthBound>,
    /// Certificates the paper must include, at most one per kind.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub certificates: Vec<CertificateReq>,
    /// What does not count toward any bound; every other part counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excludes: Option<Exclusions>,
    /// ECF event codes the paper is filed under (data, never keywords).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ecf: Vec<Ecf>,
    /// Cite holes (`cite ?("...")`, a source whose currency is
    /// unconfirmed) by field path (`cite`, `bounds/0/cite`,
    /// `excludes/cite`), with their reasons; each such cite holds its
    /// hole's reason.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub holes: BTreeMap<String, Option<String>>,
}

/// A length measure. `lines` counts lines of monospaced type (FRAP
/// 32(a)(7)(B)(i), 32(g)(1)): a paper in a proportionally spaced face has
/// no line count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    /// `words`.
    Words,
    /// `pages`.
    Pages,
    /// `lines` (monospaced).
    Lines,
}

impl Measure {
    /// The keyword spelling (`words`, `pages`, `lines`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Words => "words",
            Self::Pages => "pages",
            Self::Lines => "lines",
        }
    }
}

/// `words|pages|lines <= N`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LengthBound {
    /// What is counted.
    pub measure: Measure,
    /// The most the paper may hold; a hole judges no paper within or over.
    pub max: Authored<u64>,
    /// The entry's own cite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Source span of the entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

/// `requires certificate c [for [measures]]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateReq {
    /// The certificate kind (`compliance`).
    pub kind: String,
    /// The bounds it goes with: the paper must carry it when it relies on
    /// one of them (FRAP 32(g)(1): a brief submitted under the
    /// type-volume limitation). Empty: always.
    #[serde(default, rename = "for", skip_serializing_if = "Vec::is_empty")]
    pub measures: Vec<Measure>,
    /// The entry's own cite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Source span of the entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

/// `excludes [...]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exclusions {
    /// Excluded parts in source order; a hole stands for parts not yet
    /// identified.
    pub items: Vec<Authored<String>>,
    /// The entry's own cite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Source span of the entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl Exclusions {
    /// True when a hole stands for parts not yet identified.
    #[must_use]
    pub fn open(&self) -> bool {
        self.items.iter().any(|i| matches!(i, Authored::Hole(_)))
    }
}

/// `ecf "code"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ecf {
    /// The event code.
    pub code: Authored<String>,
    /// The entry's own cite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Source span of the entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

/// A paper's counts, each without the parts its limit excludes; `None`
/// when not counted (a paper in a proportionally spaced face has no line
/// count).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    /// Words.
    pub words: Option<u64>,
    /// Pages.
    pub pages: Option<u64>,
    /// Lines of monospaced type.
    pub lines: Option<u64>,
}

impl Counts {
    /// The count of `measure`.
    #[must_use]
    pub const fn of(&self, measure: Measure) -> Option<u64> {
        match measure {
            Measure::Words => self.words,
            Measure::Pages => self.pages,
            Measure::Lines => self.lines,
        }
    }
}

/// Whether a paper is within a bound or a limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Within.
    Within,
    /// Over.
    Over,
    /// Not decided: a hole, a count not given, or an open exclusion list
    /// under a count over the bound.
    Unknown,
}

impl Limit {
    /// `counts` against `bound`. Excluding more can only lower a count, so
    /// with an open exclusion list a count over the bound is unknown.
    #[must_use]
    pub fn bound(&self, bound: &LengthBound, counts: &Counts) -> Verdict {
        let open = self.excludes.as_ref().is_some_and(Exclusions::open);
        match (&bound.max, counts.of(bound.measure)) {
            (Authored::Known(max), Some(n)) if n <= *max => Verdict::Within,
            (Authored::Known(_), Some(_)) if !open => Verdict::Over,
            _ => Verdict::Unknown,
        }
    }

    /// `counts` against the bounds, which are alternatives: within one is
    /// within, over all is over. `None` when the limit has no bound.
    #[must_use]
    pub fn length(&self, counts: &Counts) -> Option<Verdict> {
        let verdicts: Vec<Verdict> = self.bounds.iter().map(|b| self.bound(b, counts)).collect();
        if verdicts.is_empty() {
            None
        } else if verdicts.contains(&Verdict::Within) {
            Some(Verdict::Within)
        } else if verdicts.iter().all(|v| *v == Verdict::Over) {
            Some(Verdict::Over)
        } else {
            Some(Verdict::Unknown)
        }
    }

    /// Whether a paper with `counts` must include `cert`: always when it
    /// names no measure; otherwise when the paper relies on a bound it
    /// names, within one of them and none of the others. `None` when that
    /// is not decided.
    #[must_use]
    pub fn requires(&self, cert: &CertificateReq, counts: &Counts) -> Option<bool> {
        if cert.measures.is_empty() {
            return Some(true);
        }
        let within = |named: bool| {
            self.bounds.iter().any(|b| {
                cert.measures.contains(&b.measure) == named
                    && self.bound(b, counts) == Verdict::Within
            })
        };
        if within(false) {
            Some(false)
        } else if within(true) {
            Some(true)
        } else {
            None
        }
    }
}
