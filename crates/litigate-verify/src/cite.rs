//! Cite binding and primary-source verification for the authoring compiler.
use crate::context::{spelled, Cite, GAP, GAP_WORDS};
use crate::diag::{emit, note};
use crate::Context;
use litigate_base::{Diagnostic, DiagnosticCode as C, Profile, Severity, Span};
use litigate_engine::cite::{parse_cite_string, CiteGrammar, Family, SourceCorpus};
use litigate_engine::model::Pack;
use litigate_lower::source_map::{subdivided, Citations, L0Section, Pin};
use litigate_sema::hir::DayUnit;
use litigate_sema::Hole;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;
include!("cite/quotes.rs");
include!("cite/measures.rs");
include!("cite/sources.rs");
