//! Citations in running text, in the manner of eyecite: an authority (a
//! rule set's prefix, or a title of the U.S. Code or the C.F.R.), a section
//! and its subdivisions. Prefixes are the national rule sets' and every
//! `cite_syntax` and `cite_prefixes` spelling the packages declare;
//! spellings in one group name one authority.

use regex::Regex;
use std::collections::BTreeMap;
use std::fmt;

/// Spellings of the national rule sets beyond the packages' own, as
/// opinions write them.
const NATIONAL: [&[&str]; 6] = [
    &[
        "FRAP",
        "Fed. R. App. P.",
        "Federal Rule of Appellate Procedure",
        "Federal Rules of Appellate Procedure",
    ],
    &[
        "FRCP",
        "Fed. R. Civ. P.",
        "Federal Rule of Civil Procedure",
        "Federal Rules of Civil Procedure",
    ],
    &[
        "FRCrimP",
        "Fed. R. Crim. P.",
        "Federal Rule of Criminal Procedure",
        "Federal Rules of Criminal Procedure",
    ],
    &[
        "FRBP",
        "Fed. R. Bankr. P.",
        "Federal Rule of Bankruptcy Procedure",
        "Bankruptcy Rule",
    ],
    &["FRE", "Fed. R. Evid.", "Federal Rule of Evidence"],
    &["Sup. Ct. R.", "Supreme Court Rule"],
];

/// The longest range (`§§ 141-144`) expanded section by section.
const MAX_RANGE: u64 = 50;

/// One cited provision.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cite {
    /// The authority: a prefix group's key, `N usc` or `N cfr`.
    pub authority: String,
    /// The section or rule number (`4`, `42.75`, `1295`).
    pub section: String,
    /// Its subdivisions, outermost first (`["a", "1", "A"]`).
    pub subs: Vec<String>,
}

impl Cite {
    /// Whether `self` and `other` name the same provision or one contains
    /// the other (`FRAP 4(a)` and `FRAP 4(a)(1)(A)`).
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        let n = self.subs.len().min(other.subs.len());
        self.authority == other.authority
            && self.section == other.section
            && self.subs[..n] == other.subs[..n]
    }
}

impl fmt::Display for Cite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.authority, self.section)?;
        self.subs.iter().try_for_each(|s| write!(f, "({s})"))
    }
}

/// Finds [`Cite`]s in text.
#[derive(Debug, Clone)]
pub struct Extractor {
    head: Regex,
    more: Regex,
    keys: BTreeMap<String, String>,
}

impl Extractor {
    /// An extractor knowing the national rule sets and `groups` (each a set
    /// of spellings of one authority).
    ///
    /// # Panics
    /// Never: the patterns are fixed syntax around escaped spellings.
    #[must_use]
    pub fn new(groups: &[Vec<String>]) -> Self {
        let mut all: Vec<Vec<String>> = NATIONAL
            .iter()
            .map(|g| g.iter().map(|s| (*s).to_owned()).collect())
            .collect();
        all.extend(groups.iter().cloned());
        let keys = group_keys(&all);
        let mut spellings: Vec<&String> = all.iter().flatten().collect();
        spellings.sort_by_key(|s| std::cmp::Reverse(s.len()));
        spellings.dedup();
        let prefixes = spellings
            .iter()
            .map(|s| spelling_pattern(s))
            .collect::<Vec<_>>()
            .join("|");
        let head = format!(
            r"(?:\b(?P<code>\d{{1,2}})\s+(?P<kind>U\.?\s?S\.?\s?C\.?(?:\s?A\.?)?|C\.?\s?F\.?\s?R\.?)|(?P<prefix>{prefixes}))\s*,?\s*(?:Rule\s+|Part\s+)?(?:§{{1,2}}\s*)?(?P<sec>\d+[A-Za-z]?(?:\.\d+[A-Za-z]?)*)\b(?P<subs>(?:\([A-Za-z0-9]{{1,4}}\))*)"
        );
        // Both patterns are built from escaped text and fixed syntax.
        #[allow(clippy::unwrap_used)]
        let head = Regex::new(&head).unwrap();
        #[allow(clippy::unwrap_used)]
        let more = Regex::new(
            r"^\s*(?P<sep>,\s*(?:and\b|&)?|and\b|&|-|–|through\b)\s*(?:§\s*)?(?P<sec>\d+[A-Za-z]?(?:\.\d+[A-Za-z]?)*)\b(?P<subs>(?:\([A-Za-z0-9]{1,4}\))*)",
        )
        .unwrap();
        Self { head, more, keys }
    }

    /// Every cite in `text`, in order, a list (`FRAP 10, 11`) or range
    /// (`§§ 141-144`) giving one per section.
    #[must_use]
    pub fn extract(&self, text: &str) -> Vec<Cite> {
        let mut out = Vec::new();
        for caps in self.head.captures_iter(text) {
            let authority = if let (Some(code), Some(kind)) = (caps.name("code"), caps.name("kind"))
            {
                let kind = if kind.as_str().starts_with('U') {
                    "usc"
                } else {
                    "cfr"
                };
                format!("{} {kind}", code.as_str())
            } else {
                self.keys
                    .get(&squash(&caps["prefix"]))
                    .cloned()
                    .unwrap_or_default()
            };
            let mut section = section_of(&caps["sec"]);
            out.push(Cite {
                authority: authority.clone(),
                section: section.clone(),
                subs: subs_of(&caps["subs"]),
            });
            let mut rest = &text[caps.get(0).map_or(0, |m| m.end())..];
            while let Some(next) = self.more.captures(rest) {
                let to = section_of(&next["sec"]);
                let ranged = matches!(&next["sep"], "-" | "–" | "through");
                let span = (section.parse::<u64>(), to.parse::<u64>());
                if let (true, Ok(from), Ok(end)) = (ranged, span.0, span.1) {
                    let within = end.saturating_sub(from) <= MAX_RANGE;
                    for n in (from + 1..end).filter(|_| within) {
                        out.push(Cite {
                            authority: authority.clone(),
                            section: n.to_string(),
                            subs: Vec::new(),
                        });
                    }
                }
                out.push(Cite {
                    authority: authority.clone(),
                    section: to.clone(),
                    subs: subs_of(&next["subs"]),
                });
                section = to;
                rest = &rest[next.get(0).map_or(0, |m| m.end())..];
            }
        }
        out
    }
}

/// `31.0` -> `31`: a local rule's `.0` is the rule itself.
fn section_of(raw: &str) -> String {
    raw.strip_suffix(".0").unwrap_or(raw).to_owned()
}

fn subs_of(raw: &str) -> Vec<String> {
    raw.split(['(', ')'])
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// A spelling's comparison form: letters and digits, lowercased.
fn squash(spelling: &str) -> String {
    spelling
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// A spelling as a pattern: periods optional, any spacing.
fn spelling_pattern(spelling: &str) -> String {
    let mut out = String::from(r"\b");
    for c in spelling.trim().chars() {
        match c {
            '.' => out.push_str(r"\.?"),
            c if c.is_whitespace() => out.push_str(r"\s*"),
            c => out.push_str(&regex::escape(&c.to_string())),
        }
    }
    out
}

/// Squashed spelling -> its group's key (the squashed form of the group's
/// first spelling, national groups first), groups sharing a spelling
/// merged.
fn group_keys(groups: &[Vec<String>]) -> BTreeMap<String, String> {
    fn root(parent: &BTreeMap<String, String>, s: &str) -> String {
        let mut at = s.to_owned();
        while let Some(up) = parent.get(&at).filter(|up| **up != at) {
            at.clone_from(up);
        }
        at
    }
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    let mut order: BTreeMap<String, usize> = BTreeMap::new();
    for group in groups {
        let squashed: Vec<String> = group.iter().map(|s| squash(s)).collect();
        for s in &squashed {
            let next = order.len();
            order.entry(s.clone()).or_insert(next);
            parent.entry(s.clone()).or_insert_with(|| s.clone());
        }
        for pair in squashed.windows(2) {
            let (a, b) = (root(&parent, &pair[0]), root(&parent, &pair[1]));
            let (first, later) = if order[&a] <= order[&b] {
                (a, b)
            } else {
                (b, a)
            };
            parent.insert(later, first);
        }
    }
    parent
        .keys()
        .map(|s| (s.clone(), root(&parent, s)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cites(groups: &[&[&str]], text: &str) -> Vec<String> {
        let groups: Vec<Vec<String>> = groups
            .iter()
            .map(|g| g.iter().map(|s| (*s).to_owned()).collect())
            .collect();
        Extractor::new(&groups)
            .extract(text)
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn rule_spellings_name_one_authority() {
        assert_eq!(
            cites(
                &[],
                "under Fed. R. App. P. 4(a)(1)(A) and Federal Rule of Appellate Procedure 4(b), FRAP 26"
            ),
            ["frap 4(a)(1)(A)", "frap 4(b)", "frap 26"]
        );
    }

    #[test]
    fn code_titles_sections_lists_and_ranges() {
        assert_eq!(
            cites(
                &[],
                "28 U.S.C. § 2107(a); 35 U.S.C. §§ 141-144, 319; 37 C.F.R. § 42.75(c)"
            ),
            [
                "28 usc 2107(a)",
                "35 usc 141",
                "35 usc 142",
                "35 usc 143",
                "35 usc 144",
                "35 usc 319",
                "37 cfr 42.75(c)"
            ]
        );
        assert_eq!(
            cites(&[], "FRAP 10, 11, and 12"),
            ["frap 10", "frap 11", "frap 12"]
        );
        assert_eq!(cites(&[], "8 U.S.C. § 1 - 900"), ["8 usc 1", "8 usc 900"]);
    }

    #[test]
    fn package_prefixes_join_the_groups_they_share_a_spelling_with() {
        let groups: &[&[&str]] = &[&["8th Cir. R."], &["8th Cir. R.", "Eighth Circuit Rule"]];
        assert_eq!(
            cites(
                groups,
                "Eighth Circuit Rule 28A(h) and 8th Cir. R. 47B; 1st Cir. R. 31.0"
            ),
            ["8thcirr 28A(h)", "8thcirr 47B"]
        );
        assert_eq!(
            cites(&[&["1st Cir. R."]], "1st Cir. R. 31.0"),
            ["1stcirr 31"]
        );
    }

    #[test]
    fn a_cite_overlaps_its_subdivisions_not_its_siblings() {
        let ex = Extractor::new(&[]);
        let all = ex.extract("FRAP 4(a) FRAP 4(a)(1)(A) FRAP 4(b) FRAP 40 FRCP 4(a)");
        assert!(all[0].overlaps(&all[1]) && all[1].overlaps(&all[0]));
        assert!(!all[0].overlaps(&all[2]));
        assert!(!all[0].overlaps(&all[3]));
        assert!(!all[0].overlaps(&all[4]));
    }
}
