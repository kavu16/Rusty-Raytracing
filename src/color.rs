use crate::interval::Interval;
use crate::vec3::*;

pub type Color = Vec3;

#[inline]
fn linear_to_gamma(linear_comp: f64) -> f64 {
    if linear_comp > 0.0 {
        return linear_comp.sqrt();
    }
    0.0
}

impl Color {
    pub fn write_color(Color { x: r, y: g, z: b }: Color) {
        let r = linear_to_gamma(r);
        let g = linear_to_gamma(g);
        let b = linear_to_gamma(b);
        const INTENSITY: Interval = Interval {
            min: 0.000,
            max: 0.999,
        };
        let rbyte = (256.0 * r.clamp(INTENSITY.min, INTENSITY.max)) as i32;
        let gbyte = (256.0 * g.clamp(INTENSITY.min, INTENSITY.max)) as i32;
        let bbyte = (256.0 * b.clamp(INTENSITY.min, INTENSITY.max)) as i32;

        println!("{rbyte} {gbyte} {bbyte}");
    }
}
