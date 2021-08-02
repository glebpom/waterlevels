use std::ops::Range;

use crate::Index;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Direction {
    Left,
    Right,
}

impl Direction {
    /// Modify index in the range, considering the direction.
    /// Returns false if modification would lead to out-of-range
    pub(crate) fn set_index_to_next(&self, idx: &mut Index, range: Range<usize>) -> bool {
        match self {
            Direction::Left => {
                if *idx == 0 {
                    false
                } else {
                    *idx -= 1;
                    true
                }
            }
            Direction::Right => {
                if !range.contains(&(*idx + 1)) {
                    false
                } else {
                    *idx += 1;
                    true
                }
            }
        }
    }
}