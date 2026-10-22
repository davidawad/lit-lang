//! `SemVer` versions and the four requirement operators of SPEC §3.2
//! (`^`, `~`, `>=`, `=`), with Cargo's caret/tilde semantics.
use std::cmp::Ordering;
use std::fmt;

/// `MAJOR.MINOR.PATCH[-pre]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// Major.
    pub major: u64,
    /// Minor.
    pub minor: u64,
    /// Patch.
    pub patch: u64,
    /// Dot-separated pre-release identifiers (empty for a release).
    pub pre: Vec<String>,
}

fn number(text: &str) -> Option<u64> {
    let leading_zero = text.len() > 1 && text.starts_with('0');
    (!text.is_empty() && !leading_zero && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

impl Version {
    /// Parses a strict `SemVer` version (no build metadata).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (text, None),
        };
        let mut parts = core.split('.');
        let (major, minor, patch) = (
            number(parts.next()?)?,
            number(parts.next()?)?,
            number(parts.next()?)?,
        );
        if parts.next().is_some() {
            return None;
        }
        let pre = match pre {
            None => Vec::new(),
            Some(pre) => {
                let ids: Vec<String> = pre.split('.').map(str::to_owned).collect();
                let valid = ids.iter().all(|id| {
                    !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                });
                if !valid {
                    return None;
                }
                ids
            }
        };
        Some(Self {
            major,
            minor,
            patch,
            pre,
        })
    }

    fn core(&self) -> (u64, u64, u64) {
        (self.major, self.minor, self.patch)
    }
}

fn cmp_pre(a: &[String], b: &[String]) -> Ordering {
    match (a.is_empty(), b.is_empty()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => {
            for (x, y) in a.iter().zip(b) {
                let ord = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(x), Ok(y)) => x.cmp(&y),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => x.cmp(y),
                };
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            a.len().cmp(&b.len())
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.core()
            .cmp(&other.core())
            .then_with(|| cmp_pre(&self.pre, &other.pre))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre.join("."))?;
        }
        Ok(())
    }
}

/// Requirement operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `^`: compatible (same leftmost non-zero component).
    Caret,
    /// `~`: same major and minor.
    Tilde,
    /// `>=`.
    AtLeast,
    /// `=`.
    Exact,
}

/// `op version`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// Operator.
    pub op: Op,
    /// Base version.
    pub version: Version,
}

impl Requirement {
    /// Parses `^1.2.3`, `~1.2.3`, `>=1.2.3`, or `=1.2.3` (no spaces).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (op, rest) = if let Some(rest) = text.strip_prefix(">=") {
            (Op::AtLeast, rest)
        } else if let Some(rest) = text.strip_prefix('^') {
            (Op::Caret, rest)
        } else if let Some(rest) = text.strip_prefix('~') {
            (Op::Tilde, rest)
        } else {
            (Op::Exact, text.strip_prefix('=')?)
        };
        Some(Self {
            op,
            version: Version::parse(rest)?,
        })
    }

    /// Whether `v` satisfies this requirement. A pre-release version only
    /// matches a requirement naming the same `MAJOR.MINOR.PATCH`.
    #[must_use]
    pub fn matches(&self, v: &Version) -> bool {
        let base = &self.version;
        if !v.pre.is_empty() && v.core() != base.core() {
            return false;
        }
        if v < base {
            return false;
        }
        match self.op {
            Op::Exact => v == base,
            Op::AtLeast => true,
            Op::Tilde => (v.major, v.minor) == (base.major, base.minor),
            Op::Caret if base.major > 0 => v.major == base.major,
            Op::Caret if base.minor > 0 => v.major == 0 && v.minor == base.minor,
            Op::Caret => v.core() == base.core(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn req(r: &str, v: &str) -> bool {
        match (Requirement::parse(r), Version::parse(v)) {
            (Some(r), Some(v)) => r.matches(&v),
            _ => false,
        }
    }

    #[test]
    fn caret_tilde_exact_and_at_least() {
        assert!(req("^0.1.0", "0.1.9") && !req("^0.1.0", "0.2.0"));
        assert!(req("^1.2.3", "1.9.0") && !req("^1.2.3", "2.0.0") && !req("^1.2.3", "1.2.2"));
        assert!(req("^0.0.3", "0.0.3") && !req("^0.0.3", "0.0.4"));
        assert!(req("~2024.12.0", "2024.12.5") && !req("~2024.12.0", "2024.13.0"));
        assert!(req(">=1.0.0", "7.0.0") && req("=1.0.0", "1.0.0") && !req("=1.0.0", "1.0.1"));
        assert!(!req("^1.0.0", "1.1.0-rc.1") && req("^1.1.0-rc.1", "1.1.0-rc.2"));
        assert!(Version::parse("01.0.0").is_none() && Version::parse("1.0").is_none());
        assert!(Requirement::parse("1.0.0").is_none());
    }

    proptest! {
        #[test]
        fn prop_display_round_trips(a in 0_u64..10_000, b in 0_u64..10_000, c in 0_u64..10_000, pre in proptest::option::of("[a-z]{1,4}\\.[0-9]{1,3}")) {
            let text = match &pre { Some(p) => format!("{a}.{b}.{c}-{p}"), None => format!("{a}.{b}.{c}") };
            let v = Version::parse(&text);
            prop_assert_eq!(v.map(|v| v.to_string()), Some(text));
        }

        #[test]
        fn prop_release_satisfies_its_own_requirements(a in 0_u64..50, b in 0_u64..50, c in 0_u64..50) {
            for op in ["^", "~", ">=", "="] {
                let (r, v) = (format!("{op}{a}.{b}.{c}"), format!("{a}.{b}.{c}"));
                prop_assert!(req(&r, &v));
            }
        }
    }
}
