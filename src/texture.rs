use std::sync::Arc;

use crate::{color::Color, perlin::Perlin, vec3::Point3};

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

#[derive(Clone, Copy, Default, Debug)]
pub struct SolidColor {
    albedo: Color
}

impl SolidColor {
    pub fn new(albedo: &Color) -> Self {
        Self {
            albedo: *albedo,
        }
    }
}

impl From<(f64, f64, f64)> for SolidColor {
    fn from((r, g, b): (f64, f64, f64)) -> Self {
        Self::new(&Color::new(r, g, b))
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.albedo
    }
}

unsafe impl Send for SolidColor {}
unsafe impl Sync for SolidColor {}

#[derive(Clone)]
pub struct CheckerTexture {
    inv_scale: f64,
    even: Arc<dyn Texture>,
    odd: Arc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, even: Arc<dyn Texture>, odd: Arc<dyn Texture>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }
}

impl From<(f64, &Color, &Color)> for CheckerTexture {
    fn from((scale, c1, c2): (f64, &Color, &Color)) -> Self {
        CheckerTexture::new(scale, Arc::new(SolidColor::new(c1)), Arc::new(SolidColor::new(c2)))
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let x_integer = (self.inv_scale * p.x).floor() as i32;
        let y_integer = (self.inv_scale * p.y).floor() as i32;
        let z_integer = (self.inv_scale * p.z).floor() as i32;

        if (x_integer + y_integer + z_integer) % 2 == 0 {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}

unsafe impl Send for CheckerTexture {}
unsafe impl Sync for CheckerTexture {}

pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {
        Color::new(0.5, 0.5, 0.5) * (1.0 + (self.scale * p.z + 10.0 * self.noise.turb(*p, 7)).sin()) 
    }
}

unsafe impl Send for NoiseTexture {}
unsafe impl Sync for NoiseTexture {}