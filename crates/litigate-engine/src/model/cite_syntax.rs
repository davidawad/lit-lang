// SPDX-License-Identifier: GPL-3.0-or-later
//! Pack-authored citation syntax (`Pack.citeSyntax`): the rule-body prefixes
//! and forum shorthands the cite normalizer (`cite::normalize`) recognizes
//! for one pack. The engine itself knows only the forum-neutral citation
//! shapes (U.S.C., C.F.R., case names); every rule body ("FRCP 12",
//! "RCFC 56", "MPEP § 714") and every pack-private shorthand is declared
//! here, so a new forum needs no engine change. See
//! `docs/PACK_SCHEMA.md#citation-syntax`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Citation syntax a pack's `cite`/`authority` strings are written in.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CiteSyntax {
    /// Rule/manual bodies cited by a textual prefix ("FRCP 12(b)(6)").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<CiteRuleForm>,
    /// Pack-private compact forms that imply a U.S.C./C.F.R. title
    /// (`"cfr210.16.b.4"` for 19 C.F.R. § 210.16(b)(4)).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shorthands: Vec<CiteShorthand>,
}

/// One rule body: the prefixes it is written with and the corpus heading a
/// citation of it resolves to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CiteRuleForm {
    /// Surface prefixes, matched case-insensitively at the start of a
    /// citation, one space matching any run of spaces
    /// (`["FRCP", "Fed. R. Civ. P."]`).
    pub prefixes: Vec<String>,
    /// The vendored corpus's `## <heading>` for a section of this body,
    /// with exactly one `{}` standing for the section number
    /// (`"Rule {}"`). Two forms with the same heading are the same body.
    pub heading: String,
    /// Id of the `sources[]` entry this body normalizes into, when the pack
    /// has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Which code a shorthand's implied title belongs to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CiteCode {
    /// United States Code.
    Usc,
    /// Code of Federal Regulations.
    Cfr,
}

/// A compact dotted form: `<prefix><section parts>.<subsection>...`, the
/// first part all digits (`"usc1337.b.1"`, `"cfr210.16.b.4"`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CiteShorthand {
    /// Literal prefix, matched case-insensitively (`"cfr"`).
    pub prefix: String,
    /// The code the citation belongs to.
    pub code: CiteCode,
    /// The title number the shorthand implies.
    pub title: u16,
    /// How many leading dotted parts form the section number: `2` makes
    /// `"cfr210.16.b"` section `210.16`, `1` makes `"usc1337.b"` section
    /// `1337`. Every later part is a subsection.
    pub section_parts: u8,
    /// Id of the `sources[]` entry this shorthand normalizes into, when the
    /// pack has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl CiteSyntax {
    /// Whether nothing is declared (the field is then omitted on output).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty() && self.shorthands.is_empty()
    }

    /// Reject a syntax the normalizer could not apply unambiguously:
    /// an empty prefix or heading, a heading without exactly one `{}`, a
    /// shorthand with a zero title or `sectionParts`, a `source` that is
    /// not one of `source_ids`, or two prefixes where one is a
    /// case-insensitive prefix of another (which would make the match
    /// depend on declaration order).
    ///
    /// # Errors
    /// `Error::Parse` naming the pack and the offending entry.
    pub fn validate(&self, pack_id: &str, source_ids: &[&str]) -> Result<()> {
        let bad = |what: String| Err(Error::Parse(format!("pack {pack_id}: citeSyntax {what}")));
        let check_source = |source: Option<&String>| -> Result<()> {
            match source {
                Some(s) if !source_ids.contains(&s.as_str()) => Err(Error::Parse(format!(
                    "pack {pack_id}: citeSyntax source `{s}` is not a `sources[].id` (valid: {})",
                    source_ids.join(", ")
                ))),
                _ => Ok(()),
            }
        };
        let mut prefixes: Vec<&str> = vec![];
        for form in &self.rules {
            if form.heading.matches("{}").count() != 1 {
                return bad(format!(
                    "heading `{}` must contain exactly one `{{}}`",
                    form.heading
                ));
            }
            if form.prefixes.is_empty() {
                return bad(format!("rule `{}` has no prefixes", form.heading));
            }
            check_source(form.source.as_ref())?;
            prefixes.extend(form.prefixes.iter().map(String::as_str));
        }
        for sh in &self.shorthands {
            if sh.title == 0 || sh.section_parts == 0 {
                return bad(format!(
                    "shorthand `{}` needs a nonzero title and sectionParts",
                    sh.prefix
                ));
            }
            check_source(sh.source.as_ref())?;
            prefixes.push(&sh.prefix);
        }
        if let Some(p) = prefixes.iter().find(|p| p.trim().is_empty()) {
            return bad(format!("prefix `{p}` is empty"));
        }
        for (i, a) in prefixes.iter().enumerate() {
            for b in &prefixes[i + 1..] {
                let (a_l, b_l) = (a.to_ascii_lowercase(), b.to_ascii_lowercase());
                if a_l.starts_with(&b_l) || b_l.starts_with(&a_l) {
                    return bad(format!("prefixes `{a}` and `{b}` overlap"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn syntax(v: serde_json::Value) -> CiteSyntax {
        serde_json::from_value(v).expect("syntax parses")
    }

    fn err(v: serde_json::Value) -> String {
        match syntax(v).validate("p", &["s1"]) {
            Err(Error::Parse(m)) => m,
            other => panic!("expected a parse error, got {other:?}"),
        }
    }

    #[test]
    fn accepts_well_formed_syntax() {
        let s = syntax(serde_json::json!({
            "rules": [{"prefixes": ["AB", "A. B."], "heading": "AB {}", "source": "s1"}],
            "shorthands": [{"prefix": "zz", "code": "cfr", "title": 9, "sectionParts": 2}]
        }));
        assert!(s.validate("p", &["s1"]).is_ok());
        assert!(!s.is_empty());
        assert!(CiteSyntax::default().is_empty());
    }

    #[test]
    fn rejects_heading_without_exactly_one_placeholder() {
        let m = err(serde_json::json!({"rules": [{"prefixes": ["AB"], "heading": "AB"}]}));
        assert_eq!(
            m,
            "pack p: citeSyntax heading `AB` must contain exactly one `{}`"
        );
        let m = err(serde_json::json!({"rules": [{"prefixes": ["AB"], "heading": "{} {}"}]}));
        assert!(m.contains("exactly one"), "{m}");
    }

    #[test]
    fn rejects_empty_and_overlapping_prefixes() {
        let m = err(serde_json::json!({"rules": [{"prefixes": [], "heading": "AB {}"}]}));
        assert_eq!(m, "pack p: citeSyntax rule `AB {}` has no prefixes");
        let m = err(serde_json::json!({"rules": [{"prefixes": [" "], "heading": "AB {}"}]}));
        assert_eq!(m, "pack p: citeSyntax prefix ` ` is empty");
        let m = err(serde_json::json!({"rules": [
            {"prefixes": ["AB"], "heading": "AB {}"},
            {"prefixes": ["ab c"], "heading": "ABC {}"}
        ]}));
        assert_eq!(m, "pack p: citeSyntax prefixes `AB` and `ab c` overlap");
    }

    #[test]
    fn rejects_unknown_source_and_zero_shorthand_fields() {
        let m = err(
            serde_json::json!({"rules": [{"prefixes": ["AB"], "heading": "AB {}", "source": "nope"}]}),
        );
        assert_eq!(
            m,
            "pack p: citeSyntax source `nope` is not a `sources[].id` (valid: s1)"
        );
        let m = err(serde_json::json!({"shorthands": [
            {"prefix": "zz", "code": "usc", "title": 0, "sectionParts": 1}
        ]}));
        assert_eq!(
            m,
            "pack p: citeSyntax shorthand `zz` needs a nonzero title and sectionParts"
        );
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let r: std::result::Result<CiteSyntax, _> =
            serde_json::from_value(serde_json::json!({"rule": []}));
        assert!(r.is_err());
    }
}
