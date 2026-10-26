//! A pack's filing requirements: each paper kind's length bounds,
//! certificates, exclusions and ECF event codes in force at the build
//! dates, as the litigate `limit` declaration lowers them (litigate SPEC
//! §3.7). Data for checking a filed paper; the engine's analyses do not
//! read them. A value the sources do not settle is absent here and listed
//! in `lowered.holes`.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One paper kind's filing requirements.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilingLimit {
    /// The paper kind (the `limit` declaration's id).
    pub id: String,
    /// The declaration's cite, covering every entry without its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
    /// Length bounds, at most one per measure. Several are alternatives:
    /// a paper within any one of them is within the limit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounds: Vec<FilingBound>,
    /// Certificates the paper must include.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub certificates: Vec<FilingCertificate>,
    /// Parts of the paper that count toward no bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excludes: Option<FilingExclusions>,
    /// ECF event codes the paper is filed under.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ecf: Vec<EcfCode>,
}

/// A length measure; `lines` counts lines of monospaced type.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Measure {
    /// Words.
    Words,
    /// Pages.
    Pages,
    /// Lines of monospaced type.
    Lines,
}

/// `measure <= max`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilingBound {
    /// What is counted.
    pub measure: Measure,
    /// The most a paper may hold (absent: a hole).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<u64>,
    /// The bound's own cite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
}

/// A certificate the paper must include.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilingCertificate {
    /// The certificate kind.
    pub kind: String,
    /// The bounds it goes with: required when the paper relies on one of
    /// them (empty: always).
    #[serde(default, rename = "for", skip_serializing_if = "Vec::is_empty")]
    pub measures: Vec<Measure>,
    /// The requirement's own cite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
}

/// What counts toward no bound.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilingExclusions {
    /// The parts named; parts not yet identified are holes at
    /// `.../excludes/open/<n>`.
    pub items: Vec<String>,
    /// The list's own cite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
}

/// An ECF event code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EcfCode {
    /// The code (absent: a hole).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// The code's own cite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,
}
