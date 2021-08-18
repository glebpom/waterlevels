use core::mem;
use std::cmp::Ordering;
use std::ops::Range;

use anyhow::bail;

use crate::{Height, Index};
use crate::direction::Direction;

#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    height: Height,
    merged_indices: Range<usize>,
}

impl Part {
    pub fn height(&self) -> Height {
        self.height
    }

    pub fn range(&self) -> Range<usize> {
        self.merged_indices.clone()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Parts {
    inner: Vec<Part>,
    velocities: Vec<(f64, usize)>,
    next_change: Option<(Vec<(Index, Height)>, f64)>,
}

/// Find the index to which water will get from the provided one
/// with the provided direction
///
/// Returns destination index
fn find_destination(parts: &[Part], current_idx: Index, direction: Direction) -> Option<Index> {
    if parts.len() <= current_idx {
        return None;
    }

    let mut found = None;
    let mut last_value = &parts[current_idx].height;
    let mut idx = current_idx;

    while direction.set_index_to_next(&mut idx, 0..parts.len()) {
        match parts[idx].height.partial_cmp(last_value) {
            Some(Ordering::Greater) => {
                break;
            }
            Some(Ordering::Equal) => {
                unreachable!("equal: {} == {}", parts[idx].height, last_value);
            }
            Some(Ordering::Less) => {
                last_value = &parts[idx].height;
                found = Some(idx);
            }
            None => {
                panic!()
            }
        }
    };

    found.or_else(|| {
        match direction {
            Direction::Left if current_idx == 0 && idx != current_idx => {
                Some(0)
            }
            Direction::Right if current_idx == parts.len() - 1 && idx != current_idx => {
                Some(parts.len() - 1)
            }
            _ => None
        }
    })
}

fn is_accept_water(parts: &[Part], idx: usize) -> bool {
    (idx == 0 || parts[idx - 1].height > parts[idx].height) &&
        (idx == parts.len() - 1 || parts[idx + 1].height > parts[idx].height)
}

fn calculate_filling_velocity(parts: &[Part]) -> Vec<(f64, usize)> {
    let mut velocities = vec![(0.0, 1); parts.len()];
    for (idx, Part { merged_indices: range, .. }) in parts.iter().enumerate() {
        velocities[idx].1 = range.len();
        if is_accept_water(parts, idx) {
            velocities[idx].0 += range.len() as f64;
        } else {
            let maybe_right = find_destination(parts, idx, Direction::Right);
            let maybe_left = find_destination(parts, idx, Direction::Left);

            match (maybe_left, maybe_right) {
                (Some(left), Some(right)) => {
                    velocities[left].0 += range.len() as f64 / 2.0;
                    velocities[left].1 = parts[left].merged_indices.len();

                    velocities[right].0 += range.len() as f64 / 2.0;
                    velocities[right].1 = parts[right].merged_indices.len();
                }
                (Some(left), None) => {
                    velocities[left].0 += range.len() as f64;
                    velocities[left].1 = parts[left].merged_indices.len();
                }
                (None, Some(right)) => {
                    velocities[right].0 += range.len() as f64;
                    velocities[right].1 = parts[right].merged_indices.len();
                }
                (None, None) => {}
            }
        };
    }

    velocities
}

/// Returns time and index when the next configuration change will occur
///
/// There is a guarantee that returned list of indices is sorted
///
/// Returns indices and corresponding resulting height along with the time
///
/// Index is as stored in parts slice
///
fn calculate_next_configuration_change(parts: &[Part], velocities: &[(f64, usize)]) -> Option<(Vec<(Index, Height)>, f64)> {
    let mut min_time_to_reach_nearest: Option<(Vec<(Index, Height)>, f64)> = None;
    for (idx, (merged_velocity, num_parts)) in velocities.iter().enumerate() {
        let velocity  = *merged_velocity / *num_parts as f64;
        if velocity > 0.0 {
            let left_diff = if idx > 0 && parts[idx].height < parts[idx - 1].height {
                Some(parts[idx - 1].height - parts[idx].height)
            } else {
                None
            };

            let right_diff = if idx < parts.len() - 1 && parts[idx].height < parts[idx + 1].height {
                Some(parts[idx + 1].height - parts[idx].height)
            } else {
                None
            };

            let (nearest_height_diff_around, will_be_height) = match (left_diff, right_diff) {
                (Some(left), Some(right)) if left <= right =>
                    (left, parts[idx - 1].height),
                (Some(_left), Some(right)) => (right, parts[idx + 1].height),
                (Some(left), None) => (left, parts[idx - 1].height),
                (None, Some(right)) => (right, parts[idx + 1].height),
                (None, None) => continue
            };

            let time_to_reach_nearest = nearest_height_diff_around as f64 / velocity;
            match &mut min_time_to_reach_nearest {
                Some((minimal_indices, min_known_time)) if approx::abs_diff_eq!(time_to_reach_nearest, *min_known_time, epsilon = f64::EPSILON) => {
                    minimal_indices.push((idx, will_be_height));
                }
                Some((_, min_known_time)) if time_to_reach_nearest < *min_known_time => {
                    min_time_to_reach_nearest = Some((vec![(idx, will_be_height)], time_to_reach_nearest));
                }
                None => {
                    min_time_to_reach_nearest = Some((vec![(idx, will_be_height)], time_to_reach_nearest));
                }
                _ => {}
            }
        }
    }

    min_time_to_reach_nearest
}

impl Parts {
    /// Create new Parts from the provided configuration
    ///
    /// This will join all sequential duplicates
    pub(crate) fn new(v: &[Height]) -> anyhow::Result<Self> {
        if v.is_empty() {
            bail!("should not be empty");
        }
        let mut parts = Vec::with_capacity(v.len());
        let mut cur_height = v[0];
        let mut cur_indices = 0..1;
        for (idx, part) in v.iter().enumerate() {
            if idx == 0 {
                continue;
            }

            if approx::abs_diff_eq!(*part, cur_height, epsilon = f64::EPSILON) {
                cur_indices.end += 1;
            } else {
                let indices = mem::replace(&mut cur_indices, idx..idx + 1);
                parts.push(Part { height: cur_height, merged_indices: indices });
                cur_height = *part;
            }
        }

        parts.push(Part { height: *v.last().unwrap(), merged_indices: cur_indices });

        let velocities = calculate_filling_velocity(&parts);
        let next_change = calculate_next_configuration_change(&parts, &velocities);

        Ok(Self {
            inner: parts,
            velocities,
            next_change,
        })
    }

    /// Create new Parts from the provided configuration
    ///
    /// This will join all sequential duplicates
    pub(crate) fn new_from_parts_and_changes(v: &[Part], changes: &[(Index, Height)]) -> anyhow::Result<Self> {
        if v.is_empty() {
            bail!("should not be empty");
        }

        let mut parts_collector: Vec<Option<Part>> = v.iter().map(|v| Some(v.clone())).collect();

        for (changed_idx, changed_height) in changes {
            let was_part = parts_collector[*changed_idx].take().unwrap();
            if *changed_idx >= 1 && Some(*changed_height) == v.get(*changed_idx - 1).map(|v| v.height) {
                let part_to_join_to = &mut parts_collector[*changed_idx - 1].as_mut().unwrap().merged_indices;
                assert_eq!(part_to_join_to.end, was_part.merged_indices.start);
                part_to_join_to.end = was_part.merged_indices.end;
            } else if *changed_idx + 1 < v.len() && Some(*changed_height) == v.get(*changed_idx + 1).map(|v| v.height) {
                let part_to_join_to = &mut parts_collector[*changed_idx + 1].as_mut().unwrap().merged_indices;
                assert_eq!(was_part.merged_indices.end, part_to_join_to.start);
                part_to_join_to.start = was_part.merged_indices.start;
            } else {
                unreachable!("expected a change but no change may occur");
            }
        }

        // it's still possible that near duplicates appear after merge. let's loop until all duplicates are eliminated
        loop {
            let was_len = parts_collector.len();
            parts_collector = parts_collector.into_iter().filter(|v| v.is_some()).collect();
            assert!(!parts_collector.is_empty());
            if parts_collector.len() == was_len {
                // no changes occurred
                break;
            }
            for idx in 1..parts_collector.len() {
                if parts_collector[idx - 1].is_some() &&
                    parts_collector[idx].is_some() &&
                    approx::abs_diff_eq!(parts_collector[idx - 1].as_ref().unwrap().height, parts_collector[idx].as_ref().unwrap().height, epsilon = f64::EPSILON) {
                    let right = parts_collector[idx].take().unwrap();

                    assert_eq!(parts_collector[idx - 1].as_ref().unwrap().merged_indices.end, right.merged_indices.start);
                    parts_collector[idx - 1].as_mut().unwrap().merged_indices.end = right.merged_indices.end;
                }
            }
        }

        let parts: Vec<_> = parts_collector.into_iter().flatten().collect();

        let velocities = calculate_filling_velocity(&parts);
        let next_change = calculate_next_configuration_change(&parts, &velocities);

        Ok(Self {
            inner: parts,
            velocities,
            next_change,
        })
    }

    pub(crate) fn calculate_parts_at_rel_time(&self, time: f64) -> Vec<Part> {
        let mut new_parts = self.inner.clone();

        for (new_part, (velocity, num_parts)) in new_parts.iter_mut().zip(self.velocities.iter()) {
            new_part.height += velocity * time / *num_parts as f64;
        }

        new_parts
    }

    pub(crate) fn next_change(&self) -> &Option<(Vec<(Index, Height)>, f64)> {
        &self.next_change
    }
}

impl AsRef<[Part]> for Parts {
    fn as_ref(&self) -> &[Part] {
        self.inner.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;

    #[test]
    fn test_example() {
        let parts = Parts::new(&[3.0, 1.0, 6.0, 4.0, 8.0, 9.0]).unwrap();

        assert_eq!(find_destination(parts.as_ref(), 0, Direction::Right).unwrap(), 1);
        assert!(find_destination(parts.as_ref(), 0, Direction::Left).is_none());
        assert!(!is_accept_water(parts.as_ref(), 0));

        assert!(is_accept_water(parts.as_ref(), 1));

        assert_eq!(find_destination(parts.as_ref(), 2, Direction::Right).unwrap(), 3);
        assert_eq!(find_destination(parts.as_ref(), 2, Direction::Left).unwrap(), 1);
        assert!(!is_accept_water(parts.as_ref(), 2));

        assert!(is_accept_water(parts.as_ref(), 3));

        assert!(find_destination(parts.as_ref(), 4, Direction::Right).is_none());
        assert_eq!(find_destination(parts.as_ref(), 4, Direction::Left).unwrap(), 3);
        assert!(!is_accept_water(parts.as_ref(), 4));

        assert!(find_destination(parts.as_ref(), 5, Direction::Right).is_none());
        assert_eq!(find_destination(parts.as_ref(), 5, Direction::Left).unwrap(), 3);
        assert!(!is_accept_water(parts.as_ref(), 5));

        let velocities = calculate_filling_velocity(parts.as_ref());
        assert_eq!(velocities, vec![(0.0, 1), (2.5, 1), (0.0, 1), (3.5, 1), (0.0, 1), (0.0, 1)]);

        let (next_configuration_changes, time_before_change) = calculate_next_configuration_change(parts.as_ref(), &velocities)
            .unwrap();

        assert_eq!(next_configuration_changes, vec![(3, 6.0)]);
        assert_abs_diff_eq!(time_before_change, 2.0f64 / 3.5f64);

        let next_parts = Parts::new_from_parts_and_changes(parts.as_ref(), &next_configuration_changes).unwrap();
        assert_eq!(next_parts.as_ref(), vec![
            Part {
                height: 3.0,
                merged_indices: 0..1,
            },
            Part {
                height: 1.0,
                merged_indices: 1..2,
            },
            Part {
                height: 6.0,
                merged_indices: 2..4,
            },
            Part {
                height: 8.0,
                merged_indices: 4..5,
            },
            Part {
                height: 9.0,
                merged_indices: 5..6,
            },
        ]);

        let velocities = calculate_filling_velocity(next_parts.as_ref());
        let (next_configuration_changes, time_before_change) = calculate_next_configuration_change(next_parts.as_ref(), &velocities)
            .unwrap();

        assert_eq!(next_configuration_changes, vec![(1, 3.0)]);
        assert_abs_diff_eq!(time_before_change, 2.0f64 / 6.0f64);

        let next_parts = Parts::new_from_parts_and_changes(next_parts.as_ref(), &next_configuration_changes).unwrap();
        assert_eq!(next_parts.as_ref(), vec![
            Part {
                height: 3.0,
                merged_indices: 0..2,
            },
            Part {
                height: 6.0,
                merged_indices: 2..4,
            },
            Part {
                height: 8.0,
                merged_indices: 4..5,
            },
            Part {
                height: 9.0,
                merged_indices: 5..6,
            },
        ]);

        let velocities = calculate_filling_velocity(next_parts.as_ref());

        let (next_configuration_changes, time_before_change) = calculate_next_configuration_change(next_parts.as_ref(), &velocities)
            .unwrap();

        assert_eq!(next_configuration_changes, vec![(0, 6.0)]);
        assert_abs_diff_eq!(time_before_change, 3.0f64 / 3.0f64);

        let next_parts = Parts::new_from_parts_and_changes(next_parts.as_ref(), &next_configuration_changes).unwrap();
        assert_eq!(next_parts.as_ref(), vec![
            Part {
                height: 6.0,
                merged_indices: 0..4,
            },
            Part {
                height: 8.0,
                merged_indices: 4..5,
            },
            Part {
                height: 9.0,
                merged_indices: 5..6,
            },
        ]);

        let velocities = calculate_filling_velocity(next_parts.as_ref());

        let (next_configuration_changes, time_before_change) = calculate_next_configuration_change(next_parts.as_ref(), &velocities)
            .unwrap();

        assert_eq!(next_configuration_changes, vec![(0, 8.0)]);
        assert_abs_diff_eq!(time_before_change, 1.0f64 / 3.0f64 * 4.0f64);

        let next_parts = Parts::new_from_parts_and_changes(next_parts.as_ref(), &next_configuration_changes).unwrap();
        assert_eq!(next_parts.as_ref(), vec![
            Part {
                height: 8.0,
                merged_indices: 0..5,
            },
            Part {
                height: 9.0,
                merged_indices: 5..6,
            },
        ]);

        let velocities = calculate_filling_velocity(next_parts.as_ref());

        let (next_configuration_changes, time_before_change) = calculate_next_configuration_change(next_parts.as_ref(), &velocities)
            .unwrap();

        assert_eq!(next_configuration_changes, vec![(0, 9.0)]);
        assert_abs_diff_eq!(time_before_change, 1.0f64 / 6.0f64 * 5.0);

        let next_parts = Parts::new_from_parts_and_changes(next_parts.as_ref(), &next_configuration_changes).unwrap();
        assert_eq!(next_parts.as_ref(), vec![
            Part {
                height: 9.0,
                merged_indices: 0..6,
            },
        ]);

        let velocities = calculate_filling_velocity(next_parts.as_ref());
        assert!(calculate_next_configuration_change(next_parts.as_ref(), &velocities).is_none());
    }

    #[test]
    fn test_with_duplicates() {
        let parts = Parts::new(&[3.0, 1.0, 1.0, 2.0, 2.0, 4.0]).unwrap();

        assert_eq!(find_destination(parts.as_ref(), 2, Direction::Left).unwrap(), 1);
        assert_eq!(find_destination(parts.as_ref(), 3, Direction::Left).unwrap(), 1);

        let velocities = calculate_filling_velocity(parts.as_ref());
        assert_eq!(velocities, vec![(0.0, 1), (6.0, 2), (0.0, 2), (0.0, 1)]);

        let (next_configuration_change_indices, time_before_change) = calculate_next_configuration_change(parts.as_ref(), &velocities)
            .unwrap();

        assert_eq!(next_configuration_change_indices, vec![(1, 2.0)]);
        assert_abs_diff_eq!(time_before_change, 2.0f64 / 6.0f64);
    }

    #[test]
    fn test_with_multiple_parts_reaching_configuration_change_at_the_same_time() {
        let parts = Parts::new(&[3.0, 2.0, 4.0, 3.0, 4.0]).unwrap();

        let velocities = calculate_filling_velocity(parts.as_ref());
        assert_eq!(velocities, vec![(0.0, 1), (2.5, 1), (0.0, 1), (2.5, 1), (0.0, 1)]);

        let (next_configuration_change_idx, _time_before_change) = calculate_next_configuration_change(parts.as_ref(), &velocities)
            .unwrap();

        assert_eq!(next_configuration_change_idx, vec![(1, 3.0), (3, 4.0)]);
    }

    #[test]
    fn test_single_element() {
        let parts = Parts::new(&[3.0]).unwrap();
        assert!(find_destination(parts.as_ref(), 0, Direction::Right).is_none());


        let velocities = calculate_filling_velocity(parts.as_ref());
        assert_eq!(velocities, vec![(1.0, 1)]);

        assert!(calculate_next_configuration_change(parts.as_ref(), &velocities)
            .is_none());
    }

    #[test]
    fn test_multiple_elements() {
        let parts = Parts::new(&[1.0, 1.0, 3.0]).unwrap();
        let velocities = calculate_filling_velocity(parts.as_ref());

        assert_eq!(velocities, vec![(3.0, 2), (0.0, 1)]);
    }

    #[test]
    fn test_empty() {
        assert!(Parts::new(&[]).is_err());
    }
}
