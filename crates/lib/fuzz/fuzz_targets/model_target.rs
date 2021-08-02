#![no_main]
use libfuzzer_sys::fuzz_target;
use snapview_test_lib::Model;

fuzz_target!(|inputs: Vec<f64>| {
    if let Ok(model) = Model::new(&inputs, f64::MAX) {
        for input in inputs {
            model.calculate_levels(input).unwrap();
        }
    }
});
