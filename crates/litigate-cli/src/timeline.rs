//! The timeline of a `.case` or `.lit` file (`litigate.timeline/1`,
//! docs/litigate/TIMELINE.md): events and deadlines in date order, the
//! worlds a matter's current posture forks into, and the layer each row
//! comes from.
//!
//! A matter's docket world holds its events and every obligation instance
//! replay created (a deadline the clock cannot count stays, with `due:
//! null` and the clock's reason), then the deadlines governing the options
//! of its current posture that nothing has triggered yet, then a fork row
//! at that posture. Each option of the posture is one alternate world: the
//! hypothetical event of taking it, then the deadlines governing the
//! options it leads to, which no recorded event dates (`unknown`).
//!
//! Until the change layers of lg-fq2 land, every package is one layer:
//! `base_rule` unless its authority is `LocalRule`, `StandingOrder` or
//! `CaseOrder`, and a row's provenance is the one layer declaring it,
//! except a filing limit's (lg-962.10.3): an event filing a paper kind
//! (`event E() paper k`) is followed by a `limit` row for the version of
//! `k` precedence selects over the matter's law, its provenance every
//! layer selection passed through, base first, so its `layer` is the one
//! that set the limit.
use crate::cmd::filing::paper;
use crate::hirx::{self, local};
use crate::matter::{CaseCheck, Scheduled};
use litigate_base::timeline::{
    ChoiceKind, EdgeKind, LayerClass, RowKind, RowState, Timeline, TimelineChoice,
    TimelineDiagnostic, TimelineEdge, TimelineEffective, TimelineLayer, TimelineRow,
    TimelineSeverity, TimelineSource, TimelineSourceKind, TimelineSpan, TimelineStatus,
    TimelineWorld, TIMELINE_SCHEMA,
};
use litigate_base::{Diagnostic, DiagnosticCode, Severity, Span};
use litigate_lower::source_map::Citations;
use litigate_sema::case::{Effect, Status};
use litigate_sema::hir::{self, DeadlineWindow, Declaration, NodeKind};
use std::collections::{BTreeMap, BTreeSet};

include!("timeline/model.rs");
include!("timeline/builder.rs");
include!("timeline/obligations.rs");
include!("timeline/events.rs");
include!("timeline/worlds.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matter;
    use crate::testkit::{self, ACME};
    use crate::ws::Session;
    use litigate_base::DiagnosticCode;
    use serde_json::json;

    include!("timeline/tests_rows.rs");
    include!("timeline/tests_worlds.rs");
}
