//! Salsa database: inputs (source files, `Lit.toml`, the workspace) and the
//! tracked queries from parse to HIR. Public APIs take `&dyn
//! salsa::Database`; no salsa type appears in HIR.
#[allow(missing_docs)] // salsa-generated constructors and setters
pub mod inputs;
pub mod parse;
pub mod queries;
pub mod resolution;
pub mod span;

use std::sync::{Arc, Mutex, PoisonError};

/// Shared log of executed queries (`query(Id(..))` debug strings).
pub type EventLog = Arc<Mutex<Vec<String>>>;

/// The litigate database.
#[salsa::db]
#[derive(Clone, Default)]
pub struct LitDatabase {
    storage: salsa::Storage<Self>,
    log: Option<EventLog>,
}

#[salsa::db]
impl salsa::Database for LitDatabase {}

impl LitDatabase {
    /// A database that records every query execution (`WillExecute`) into
    /// the returned log; used to test incrementality.
    #[must_use]
    pub fn with_event_log() -> (Self, EventLog) {
        let log: EventLog = Arc::default();
        let sink = Arc::clone(&log);
        let storage = salsa::Storage::new(Some(Box::new(move |event: salsa::Event| {
            if let salsa::EventKind::WillExecute { database_key } = event.kind {
                sink.lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(format!("{database_key:?}"));
            }
        })));
        (
            Self {
                storage,
                log: Some(Arc::clone(&log)),
            },
            log,
        )
    }

    /// Drains the event log (empty without one).
    #[must_use]
    pub fn take_log(&self) -> Vec<String> {
        self.log
            .as_ref()
            .and_then(|l| l.lock().ok().map(|mut e| std::mem::take(&mut *e)))
            .unwrap_or_default()
    }
}
