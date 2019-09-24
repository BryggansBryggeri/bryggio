use std::f32;
use std::fs::File;
use std::io::prelude::*;

pub fn read_file_to_string(file_name: &str) -> std::io::Result<String> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn f32_almost_equal(a: f32, b: f32) -> bool {
    let abs_a = a.abs();
    let abs_b = b.abs();
    let diff = (a - b).abs();

    if a == b {
        // Handle infinities.
        true
    } else if a == 0.0 || b == 0.0 || diff < f32::MIN_POSITIVE {
        // One of a or b is zero (or both are extremely close to it,) use absolute error.
        diff < (f32::EPSILON * f32::MIN_POSITIVE)
    } else {
        // Use relative error.
        (diff / f32::min(abs_a + abs_b, f32::MAX)) < f32::EPSILON
    }
}
