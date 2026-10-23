//! Content-hash caching in front of a backend.
#![allow(clippy::unwrap_used)]
use litigate_smt::{Backend, Cached, ContentHash, Error, Formula, Outcome, Query, Rational, Z3};

fn query(bound: i64) -> Query {
    let mut q = Query::new();
    let x = q.int("x").unwrap();
    q.assert("lo", &Formula::ge(&x, bound)).unwrap();
    q.assert("hi", &Formula::le(&x, bound)).unwrap();
    q
}

#[test]
fn equal_questions_hit_the_cache() {
    let mut cache = Cached::new(Z3::new().unwrap());
    assert!(cache.is_empty());
    let first = cache.check(&query(3)).unwrap();
    assert_eq!(
        first,
        Outcome::Sat([("x".to_owned(), Rational::int(3))].into())
    );
    // A separately built but identical query is the same key.
    assert_eq!(cache.check(&query(3)).unwrap(), first);
    assert_eq!((cache.hits(), cache.misses(), cache.len()), (1, 1, 1));
    assert_eq!(cache.get(&cache.key(&query(3))), Some(&first));

    cache.check(&query(4)).unwrap();
    assert_eq!((cache.hits(), cache.misses(), cache.len()), (1, 2, 2));
    assert_eq!(cache.fingerprint(), Z3::new().unwrap().fingerprint());
}

#[test]
fn key_covers_query_and_solver_configuration() {
    let fp = Z3::new().unwrap().fingerprint();
    let key = ContentHash::of(&fp, &query(3));
    assert_eq!(key, ContentHash::of(&fp, &query(3)));
    assert_ne!(key, ContentHash::of(&fp, &query(4)));
    assert_ne!(
        key,
        ContentHash::of(&Z3::with_rlimit(1).unwrap().fingerprint(), &query(3))
    );
    // Normalisation: `x >= 3` and `3 <= x` are one question.
    let mut flipped = Query::new();
    let x = flipped.int("x").unwrap();
    flipped.assert("lo", &Formula::le(3, &x)).unwrap();
    flipped.assert("hi", &Formula::ge(3, &x)).unwrap();
    assert_eq!(key, ContentHash::of(&fp, &flipped));
    // Hex rendering of the 32-byte digest.
    let hex = key.to_string();
    assert_eq!(hex.len(), 64);
    assert_eq!(&hex[..2], format!("{:02x}", key.as_bytes()[0]));
}

#[test]
fn key_is_stable_across_releases() {
    // A golden key: changing the canonical text or preimage layout must
    // be deliberate (bump KEY_FORMAT), since persisted keys would move.
    assert_eq!(
        ContentHash::of("fixed", &query(3)).to_string(),
        "414604066a2e0f621b4890144ffe4bdf75e5c1d40ddd846e9dc9fd1d7fa5a845"
    );
}

/// A backend that fails on its first attempt and answers `unknown` after.
struct Flaky(u32);

impl Backend for Flaky {
    fn fingerprint(&self) -> String {
        "flaky".into()
    }

    fn check(&mut self, _: &Query) -> Result<Outcome, Error> {
        self.0 += 1;
        if self.0 == 1 {
            Err(Error::Solver("attempt 1".into()))
        } else {
            Ok(Outcome::Unknown(format!("attempt {}", self.0)))
        }
    }
}

#[test]
fn errors_are_not_cached() {
    let mut cache = Cached::new(Flaky(0));
    assert_eq!(
        cache.check(&query(1)),
        Err(Error::Solver("attempt 1".into()))
    );
    assert!(cache.is_empty());
    let retried = Outcome::Unknown("attempt 2".into());
    assert_eq!(cache.check(&query(1)), Ok(retried.clone()));
    assert_eq!(cache.check(&query(1)), Ok(retried));
    assert_eq!((cache.hits(), cache.misses(), cache.len()), (1, 2, 1));
}
