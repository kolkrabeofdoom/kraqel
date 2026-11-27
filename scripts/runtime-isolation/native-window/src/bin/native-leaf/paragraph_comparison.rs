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

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Difference {
    pub index: usize,
    pub count: usize,
    pub local: Range<usize>,
    pub remote: Range<usize>,
    pub hint: &'static str,
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

    pub fn has_selection(&self) -> bool {
        self.current.is_some()
    }

    pub fn reset(&mut self) {
        self.current = None;
    }

    pub fn selected_service_block(&self, draft: &str) -> Option<(String, Range<usize>)> {
        let index = self.current?;
        let mut left: Vec<_> = draft.split('\n').collect();
        let right: Vec<_> = self.text.split('\n').collect();
        let (local, remote) = gaps(&left, &right).into_iter().nth(index)?;
        let start = local.start;
        let end = start + remote.len();
        left.splice(local, right[remote].iter().copied());
        let selection = field_range(&left, start..end);
        Some((left.join("\n"), selection))
    }

    pub fn step(&mut self, draft: &str, backwards: bool) -> Option<Difference> {
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
        // An empty paragraph has a zero-width selection but still occupies a paragraph.
        let hint = if b.is_empty() {
            "F8 entfernt lokale Absätze"
        } else if a.is_empty() {
            "F8 fügt Dienstabsätze ein"
        } else {
            "F8 ersetzt lokale Absätze"
        };
        let b = field_range(&right, b.clone());
        Some(Difference {
            index: index + 1,
            count,
            local: field_range(&left, a.clone()),
            remote: self.header_units + b.start..self.header_units + b.end,
            hint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_insertions_and_removals_keep_the_shared_paragraphs_aligned() {
        let mut compare = Comparison::new("🐦\nB\nC\nNeu", "X\r\n");
        let draft = "🐦\nLokal\nB\nC";
        assert_eq!(
            compare.step(draft, false),
            Some(Difference {
                index: 1,
                count: 2,
                local: 4..9,
                remote: 7..7,
                hint: "F8 entfernt lokale Absätze"
            })
        );
        assert_eq!(
            compare.selected_service_block(draft),
            Some(("🐦\nB\nC".to_owned(), 4..4))
        );
        assert_eq!(
            compare.step(draft, false),
            Some(Difference {
                index: 2,
                count: 2,
                local: 15..15,
                remote: 13..16,
                hint: "F8 fügt Dienstabsätze ein"
            })
        );
        assert_eq!(
            compare.selected_service_block(draft),
            Some(("🐦\nLokal\nB\nC\nNeu".to_owned(), 17..20))
        );
        assert_eq!(
            compare.step(draft, true),
            Some(Difference {
                index: 1,
                count: 2,
                local: 4..9,
                remote: 7..7,
                hint: "F8 entfernt lokale Absätze"
            })
        );
        assert_eq!(compare.step("🐦\nB\nC\nNeu", false), None);
    }
}
