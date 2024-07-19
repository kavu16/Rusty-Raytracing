use rand::{thread_rng, Rng};

pub const PI: f64 = 3.1415926535897932385;

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
