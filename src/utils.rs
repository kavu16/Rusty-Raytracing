use std::f64::consts::PI;

use rand::{thread_rng, Rng};

#[inline]
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

#[inline]
pub fn random_double() -> f64 {
    thread_rng().gen_range(0.0..1.0)
}

#[inline]
pub fn random_range(min: f64, max: f64) -> f64 {
    thread_rng().gen_range(min..max)
}
