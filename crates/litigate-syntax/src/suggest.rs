//! Nearest-spelling suggestions for misspelled keywords and suffixes.

/// Levenshtein edit distance over Unicode scalars.
#[must_use]
pub fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let substitute = diagonal + usize::from(ca != *cb);
            diagonal = row[j + 1];
            row[j + 1] = substitute.min(row[j] + 1).min(diagonal + 1);
        }
    }
    row[b.len()]
}

/// The unique candidate within edit distance 2 (and less than the word's
/// length) of `word`, if exactly one candidate is nearest.
#[must_use]
pub fn nearest<'a>(word: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let limit = 2.min(word.chars().count().saturating_sub(1));
    let mut scored: Vec<(usize, &'a str)> = candidates
        .iter()
        .map(|candidate| (distance(word, candidate), *candidate))
        .filter(|(score, candidate)| *score > 0 && *score <= limit && *candidate != word)
        .collect();
    scored.sort_unstable();
    scored.dedup();
    match scored.as_slice() {
        [(best, word), rest @ ..] if rest.first().is_none_or(|(next, _)| next > best) => Some(word),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{distance, nearest};

    #[test]
    fn distance_counts_edits() {
        assert_eq!(distance("stat", "state"), 1);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("kitten", "sitting"), 3);
    }

    #[test]
    fn nearest_requires_a_unique_close_match() {
        assert_eq!(nearest("stat", &["state", "start", "test"]), None);
        assert_eq!(nearest("chanse", &["chance", "state"]), Some("chance"));
        assert_eq!(nearest("xyzzy", &["chance", "state"]), None);
        assert_eq!(nearest("a", &["b"]), None);
    }
}
