//! Answers cached by a content hash of the question.
use std::collections::HashMap;
use std::fmt;

use sha2::{Digest, Sha256};

use crate::{Backend, Error, Outcome, Query};

/// Version of the hashed preimage layout; bump on any change to it.
const KEY_FORMAT: &str = "litigate-smt cache v1";

/// SHA-256 of a query's canonical SMT-LIB text together with the
/// backend's fingerprint (solver, version, options).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContentHash([u8; 32]);

impl ContentHash {
    /// The key for asking `backend_fingerprint` the question `query`.
    #[must_use]
    pub fn of(backend_fingerprint: &str, query: &Query) -> Self {
        let mut h = Sha256::new();
        for part in [KEY_FORMAT, backend_fingerprint, &query.to_smtlib()] {
            // Length-prefixed, so no two part lists share a preimage.
            h.update((part.len() as u64).to_le_bytes());
            h.update(part.as_bytes());
        }
        Self(h.finalize().into())
    }

    /// The raw digest.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|b| write!(f, "{b:02x}"))
    }
}

/// A [`Backend`] that remembers answers by [`ContentHash`].
///
/// Sound because backends are deterministic and the key covers both the
/// exact question and the solver configuration. Errors are not cached.
#[derive(Debug)]
pub struct Cached<B> {
    backend: B,
    entries: HashMap<ContentHash, Outcome>,
    hits: u64,
    misses: u64,
}

impl<B: Backend> Cached<B> {
    /// An empty cache in front of `backend`.
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            entries: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// The cache key for `query` against this backend.
    pub fn key(&self, query: &Query) -> ContentHash {
        ContentHash::of(&self.backend.fingerprint(), query)
    }

    /// The cached answer for `key`, if any.
    #[must_use]
    pub fn get(&self, key: &ContentHash) -> Option<&Outcome> {
        self.entries.get(key)
    }

    /// Number of cached answers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is cached.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Checks answered from the cache.
    #[must_use]
    pub const fn hits(&self) -> u64 {
        self.hits
    }

    /// Checks that reached the backend.
    #[must_use]
    pub const fn misses(&self) -> u64 {
        self.misses
    }
}

impl<B: Backend> Backend for Cached<B> {
    fn fingerprint(&self) -> String {
        self.backend.fingerprint()
    }

    fn check(&mut self, query: &Query) -> Result<Outcome, Error> {
        let key = self.key(query);
        if let Some(hit) = self.entries.get(&key) {
            self.hits += 1;
            return Ok(hit.clone());
        }
        self.misses += 1;
        let outcome = self.backend.check(query)?;
        self.entries.insert(key, outcome.clone());
        Ok(outcome)
    }
}
