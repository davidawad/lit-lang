//! Nearest-match suggestions for unresolved names.

/// Levenshtein distance over Unicode scalars.
#[must_use]
pub fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = (prev + usize::from(ca != *cb)).min(row[j] + 1).min(cur + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

/// The closest candidate within `max(1, len/3)` edits; ties break
/// alphabetically so the answer is deterministic.
#[must_use]
pub fn nearest<'a>(name: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let limit = (name.chars().count() / 3).max(1);
    candidates
        .into_iter()
        .filter(|c| *c != name)
        .map(|c| (distance(name, c), c))
        .filter(|(d, _)| *d <= limit)
        .min()
        .map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn finds_close_names_only() {
        assert_eq!(
            nearest("answr-due", ["answer-due", "reply"]),
            Some("answer-due")
        );
        assert_eq!(nearest("zzz", ["answer-due"]), None);
        assert_eq!(nearest("ab", ["ac", "aa"]), Some("aa"));
    }

    proptest! {
        #[test]
        fn prop_distance_is_a_metric(a in "[a-c]{0,6}", b in "[a-c]{0,6}", c in "[a-c]{0,6}") {
            prop_assert_eq!(distance(&a, &b), distance(&b, &a));
            prop_assert_eq!(distance(&a, &a), 0);
            prop_assert!(distance(&a, &c) <= distance(&a, &b) + distance(&b, &c));
        }
    }
}
