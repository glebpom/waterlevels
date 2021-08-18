pub use model::Model;
pub use parts::Part;

mod parts;
mod direction;
mod model;

type Height = f64;
type Index = usize;

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn invariant_always_met(parts: Vec<u32>, time: u32, max_time: u32) -> TestResult {
        if time > max_time {
            return TestResult::discard();
        }

        let parts: Vec<_> = parts.into_iter().map(|p| p as f64 / 100.0).collect();
        let time = time as f64 / 100.0;
        let max_time = max_time as f64 / 100.0;

        let initial_sum: f64 = parts.iter().copied().sum();
        let num_parts = parts.len() as f64;

        if let Ok(model) = Model::new(&parts, max_time) {
            let result = model.calculate_levels(time).expect("error calculating levels");
            let resulting_sum: f64 = result.iter().copied().sum();

            let calculated_amount_of_water = resulting_sum - initial_sum;
            let expected_amount_of_water = time * num_parts;

            let is_equal = approx::abs_diff_eq!(calculated_amount_of_water, expected_amount_of_water, epsilon = 0.01);

            if is_equal {
                TestResult::passed()
            } else {
                TestResult::error(format!("{} - {} ({}) != {} * {} ({})", resulting_sum, initial_sum, calculated_amount_of_water, time, num_parts, expected_amount_of_water))
            }
        } else {
            return TestResult::discard();
        }
    }
}