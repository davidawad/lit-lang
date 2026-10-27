//! Calibration sets (`calibration id: "Title" { ... }`): values measured
//! from a named source that a matter opts into by id (`analysis {
//! calibration: [...] }`), each replacing a package's own estimate.
use super::graph::TriangularDuration;
use super::logic::Metadata;
use super::value::Rational;
use serde::{Deserialize, Serialize};

/// A calibration set (`CalibrationSet`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Calibration {
    /// Declaration metadata: `id` is the set id, `label` its title.
    pub meta: Metadata,
    /// `note`: what the set covers and where it came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The sources its values come from, in declaration order.
    pub sources: Vec<CalibrationSource>,
    /// The calibrated values, in declaration order.
    pub entries: Vec<CalibrationEntry>,
}

/// `source id "Title" { url, vintage, retrieved }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalibrationSource {
    /// Name the entries cite it by.
    pub id: String,
    /// Document title.
    pub title: String,
    /// Official URL.
    pub url: String,
    /// First day of the reporting period.
    pub from: String,
    /// Day after the reporting period ends (half-open, like every
    /// interval); `None` when the source states no end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Date the source was retrieved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved: Option<String>,
}

/// What one entry sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "target", rename_all = "snake_case")]
pub enum CalibratedValue {
    /// `p edge q`: an edge's probability; `None` (`p edge ?`) clears it.
    Probability {
        /// The probability, in [0, 1].
        value: Option<Rational>,
    },
    /// `duration edge D`: an edge's elapsed time, in calendar days.
    Duration {
        /// The estimate.
        value: TriangularDuration,
    },
    /// `attr node name x`: one numeric attribute of a node.
    Attr {
        /// Attribute name.
        name: String,
        /// The value.
        value: Rational,
    },
}

/// One calibrated value (`CalibrationEntry`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationEntry {
    /// The engine ref it sets: `<pack-id>::<local>` or
    /// `<base>@<origin>::<local>`.
    pub target: String,
    /// The value.
    pub value: CalibratedValue,
    /// The [`CalibrationSource`] it was measured from.
    pub source: String,
    /// Sample size behind the value (positive).
    pub n: u64,
    /// The arithmetic deriving the value from the source's counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// The entry's source span.
    pub span: Option<litigate_base::Span>,
}
