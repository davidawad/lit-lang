//! Source identity and byte-offset to line/column mapping.
use serde::{Deserialize, Serialize};

/// Stable identifier for a source file, normally a repository-relative path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileId(pub String);

impl From<&str> for FileId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Maps UTF-8 byte offsets to one-based line and column positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    starts: Vec<usize>,
    len: usize,
    source: String,
}

impl LineIndex {
    /// Builds an index for `source`.
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(offset + 1);
            }
        }
        Self {
            starts,
            len: source.len(),
            source: source.to_owned(),
        }
    }

    /// Returns one-based `(line, column)` for a byte offset, clamping at EOF.
    #[must_use]
    pub fn line_col(&self, offset: usize) -> (u32, u32) {
        let offset = offset.min(self.len);
        let line_idx = self
            .starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1);
        let start = self.starts[line_idx];
        let valid_offset = (start..=offset)
            .rev()
            .find(|candidate| self.source.is_char_boundary(*candidate))
            .map_or(start, |boundary| boundary);
        let column = self.source[start..valid_offset].chars().count() + 1;
        (
            u32::try_from(line_idx + 1).map_or(u32::MAX, |value| value),
            u32::try_from(column).map_or(u32::MAX, |value| value),
        )
    }

    /// Returns the byte offset of the beginning of a one-based line.
    #[must_use]
    pub fn line_start(&self, line: u32) -> Option<usize> {
        let index = usize::try_from(line.checked_sub(1)?).ok()?;
        self.starts.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::LineIndex;
    use proptest::prelude::*;
    #[test]
    fn maps_lines_and_clamps_eof() {
        let index = LineIndex::new("ab\nxyz");
        assert_eq!(index.line_col(3), (2, 1));
        assert_eq!(index.line_col(99), (2, 4));
        assert_eq!(index.line_start(2), Some(3));
    }

    #[test]
    fn columns_count_unicode_scalars() {
        let index = LineIndex::new("éx");
        assert_eq!(index.line_col("é".len()), (1, 2));
    }

    proptest! {
        #[test]
        fn valid_character_boundaries_have_one_based_positions(source in ".{0,128}") {
            let index = LineIndex::new(&source);
            for (offset, _) in source.char_indices() {
                let (line, column) = index.line_col(offset);
                prop_assert!(line >= 1);
                prop_assert!(column >= 1);
                prop_assert!(index.line_start(line).is_some());
            }
        }
    }
}
