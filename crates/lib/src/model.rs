use core::iter;
use std::cmp::Ordering;

use anyhow::bail;

use crate::Height;
use crate::parts::{Parts};

#[derive(Debug)]
pub struct Model {
    /// generations represent the transition
    /// to another "merged" parts, when levels of neighbours
    /// become equal
    initial_parts: Parts,

    max_time: f64,

    generations: Vec<((f64, f64), Parts)>,
}

impl Model {
    fn calculate_generations(&mut self) -> anyhow::Result<()> {
        let mut last_generation = (self.initial_parts.clone(), 0.0);

        loop {
            match last_generation.0.next_change() {
                Some((change_indices, will_change_in)) => {
                    let end_time = last_generation.1 + will_change_in;
                    self.generations.push((
                        (last_generation.1, end_time),
                        last_generation.0.clone(),
                    ));

                    let last_state = last_generation.0.calculate_parts_at_rel_time(*will_change_in);

                    last_generation = (
                        Parts::new_from_parts_and_changes(&last_state, change_indices)?,
                        end_time
                    );
                }
                None => {
                    // final part
                    self.generations.push((
                        (last_generation.1,f64::MAX),
                        last_generation.0,
                    ));

                    break;
                }
            }
        }

        Ok(())
    }

    pub fn new(v: &[Height], max_time: f64) -> anyhow::Result<Self> {
        if v.iter().any(|item| {
            item.is_infinite() || item.is_nan() || item.is_sign_negative()
        }) {
            bail!("should be a positive number");
        }

        let mut obj = Model {
            initial_parts: Parts::new(v)?,
            generations: Vec::new(),
            max_time,
        };

        obj.calculate_generations()?;

        Ok(obj)
    }


    pub fn calculate_levels(&self, time: f64) -> anyhow::Result<Vec<Height>> {
        if time.is_sign_negative() {
            bail!("time should not be negative");
        }

        if time > self.max_time {
            bail!("more then max time time provided");
        }


        let idx = self.generations.binary_search_by(|((probe_left, probe_right), _probe)| {
            if time < *probe_left {
                Ordering::Greater
            } else if time > *probe_right {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }).unwrap();

        let ((segment_left, _), parts) = self.generations.get(idx).unwrap();

        let offset = time - segment_left;
        assert!(offset >= 0.0);

        Ok(parts.calculate_parts_at_rel_time(offset)
            .into_iter()
            .map(|part| {
                iter::repeat(part.height()).take(part.range().len())
            })
            .flatten()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_basic() {
        let model = Model::new(&[3.0, 1.0, 6.0, 4.0, 8.0, 9.0], 20.0).unwrap();
        assert_eq!(model.generations.len(), 6);

        model.calculate_levels(0.0).unwrap();
    }


    #[test]
    fn test_duplicates_after_merge_collapsing() {
        Model::new(&[0.0, 2.0, 2.0, 1.0, 2.0], 20.0).unwrap();
    }

    #[test]
    fn test_sequential_elements() {
        let model = Model::new(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], 5.0).unwrap();
        let r = model.calculate_levels(5.0).unwrap();
        for item in r {
            assert_abs_diff_eq!(item, 9.0);
        }
    }
}