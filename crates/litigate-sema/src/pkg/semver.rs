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
        let parts: Vec<&str> = core.split('.').collect();
        let [major, minor, patch] = parts[..] else {
            return None;
        };
        let (major, minor, patch) = (number(major)?, number(minor)?, number(patch)?);
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
        let pre = if self.pre.is_empty() {
            String::new()
        } else {
            format!("-{}", self.pre.join("."))
        };
        write!(f, "{}.{}.{}{pre}", self.major, self.minor, self.patch)
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
        Requirement::parse(r)
            .zip(Version::parse(v))
            .is_some_and(|(r, v)| r.matches(&v))
    }

    fn version(text: &str) -> Version {
        Version::parse(text).expect("a valid version")
    }

    #[test]
    fn caret_tilde_exact_and_at_least() {
        for (r, v, matches) in [
            ("^0.1.0", "0.1.9", true),
            ("^0.1.0", "0.2.0", false),
            ("^0.1.0", "1.1.0", false),
            ("^1.2.3", "1.9.0", true),
            ("^1.2.3", "2.0.0", false),
            ("^1.2.3", "1.2.2", false),
            ("^0.0.3", "0.0.3", true),
            ("^0.0.3", "0.0.4", false),
            ("~2024.12.0", "2024.12.5", true),
            ("~2024.12.0", "2024.13.0", false),
            (">=1.0.0", "7.0.0", true),
            ("=1.0.0", "1.0.0", true),
            ("=1.0.0", "1.0.1", false),
            ("^1.0.0", "1.1.0-rc.1", false),
            ("^1.1.0-rc.1", "1.1.0-rc.2", true),
            ("^1.1.0-rc.1", "1.1.0", true),
            ("1.0.0", "1.0.0", false),
            ("^1.0", "1.0.0", false),
        ] {
            assert_eq!(req(r, v), matches, "{r} vs {v}");
        }
    }

    #[test]
    fn versions_are_strict_semver() {
        for bad in [
            "1.0",
            "1.0.0.0",
            "01.0.0",
            "1.00.0",
            "x.0.0",
            "0.x.0",
            "0.0.x",
            "1..0",
            "1.0.0-",
            "1.0.0-rc..1",
            "1.0.0-rc_1",
            "1.0.0-rc.+",
        ] {
            assert_eq!(Version::parse(bad), None, "{bad}");
        }
        assert_eq!(version("0.10.200-rc-1.2").pre, ["rc-1", "2"]);
    }

    #[test]
    fn pre_releases_order_before_their_release() {
        let ordered = [
            "1.0.0-1",
            "1.0.0-2",
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0",
            "1.0.1",
        ];
        for pair in ordered.windows(2) {
            assert!(version(pair[0]) < version(pair[1]), "{pair:?}");
            assert!(version(pair[1]) > version(pair[0]), "{pair:?}");
        }
        assert_eq!(
            version("1.0.0-rc.1").cmp(&version("1.0.0-rc.1")),
            Ordering::Equal
        );
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
