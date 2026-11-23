use std::collections::BTreeMap;
use std::ops::Range;

// Match equal paragraphs in order. At a mismatch, resynchronize at the nearest
// common paragraph. Each returned pair is one contiguous gap between matches.
fn gaps(left: &[&str], right: &[&str]) -> Vec<(Range<usize>, Range<usize>)> {
    let mut positions: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (index, line) in right.iter().enumerate() {
        positions.entry(line).or_default().push(index);
    }
    let (mut i, mut j) = (0, 0);
    let mut result = Vec::new();
    while i < left.len() || j < right.len() {
        if i < left.len() && j < right.len() && left[i] == right[j] {
            i += 1;
            j += 1;
            continue;
        }
        let (mut next_i, mut next_j) = (left.len(), right.len());
        let mut distance = usize::MAX;
        for (skip, line) in left[i..].iter().enumerate() {
            if skip >= distance {
                break;
            }
            if let Some(indices) = positions.get(line) {
                if let Some(&k) = indices.get(indices.partition_point(|&k| k < j)) {
                    let cost = skip + (k - j);
                    if cost < distance {
                        (next_i, next_j) = (i + skip, k);
                        distance = cost;
                    }
                }
            }
        }
        result.push((i..next_i, j..next_j));
        (i, j) = (next_i, next_j);
    }
    result
}

// EDIT uses CRLF and UTF-16 offsets. Empty ranges point to the insertion
// position, including the position immediately after the last character.
fn field_range(lines: &[&str], paragraphs: Range<usize>) -> Range<usize> {
    let offset = |index| {
        lines[..index]
            .iter()
            .map(|line| line.encode_utf16().count())
            .sum::<usize>()
            + 2 * index.min(lines.len() - 1)
    };
    let start = offset(paragraphs.start);
    let end = if paragraphs.is_empty() {
        start
    } else {
        let last = paragraphs.end - 1;
        offset(last) + lines[last].encode_utf16().count()
    };
    start..end
}

pub(super) struct Comparison {
    text: String,
    header_units: usize,
    current: Option<usize>,
}

impl Comparison {
    pub fn new(text: &str, header: &str) -> Self {
        Self {
            text: text.to_owned(),
            header_units: header.encode_utf16().count(),
            current: None,
        }
    }

    pub fn reset(&mut self) {
        self.current = None;
    }

    pub fn step(
        &mut self,
        draft: &str,
        backwards: bool,
    ) -> Option<(usize, usize, Range<usize>, Range<usize>)> {
        let left: Vec<_> = draft.split('\n').collect();
        let right: Vec<_> = self.text.split('\n').collect();
        let gaps = gaps(&left, &right);
        let count = gaps.len();
        if count == 0 {
            self.reset();
            return None;
        }
        let index = match self.current {
            Some(i) if backwards => (i + count - 1) % count,
            Some(i) => (i + 1) % count,
            None if backwards => count - 1,
            None => 0,
        };
        self.current = Some(index);
        let (a, b) = &gaps[index];
        let b = field_range(&right, b.clone());
        Some((
            index + 1,
            count,
            field_range(&left, a.clone()),
            self.header_units + b.start..self.header_units + b.end,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_insertions_and_removals_keep_the_shared_paragraphs_aligned() {
        let mut compare = Comparison::new("🐦\nB\nC\nNeu", "X\r\n");
        let draft = "🐦\nLokal\nB\nC";
        assert_eq!(compare.step(draft, false), Some((1, 2, 4..9, 7..7)));
        assert_eq!(compare.step(draft, false), Some((2, 2, 15..15, 13..16)));
        assert_eq!(compare.step(draft, true), Some((1, 2, 4..9, 7..7)));
        assert_eq!(compare.step("🐦\nB\nC\nNeu", false), None);
    }
}
