use std::sync::Arc;

use crate::{
    color::Color,
    primitive::HitRecord,
    ray::Ray,
    texture::Texture,
    utils::random_double,
    vec3::{Point3, Vec3},
};

#[derive(Clone)]
pub enum Material {
    Lambertian { tex: Arc<dyn Texture> },
    Metal { albedo: Color, fuzz: f64 },
    Dielectric { refraction_index: f64 },
    DiffuseLight { tex: Arc<dyn Texture> },
}

impl Material {
    pub fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            Self::Lambertian { tex } => {
                let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

                if scatter_direction.near_zero() {
                    scatter_direction = rec.normal;
                }

                Some((
                    Ray::new(rec.p, scatter_direction, r_in.time()),
                    tex.value(rec.u, rec.v, &rec.p),
                ))
            }
            Self::Metal { albedo, fuzz } => {
                let fuzz = fuzz.min(1.0);
                let reflected = r_in.direction().reflect(&rec.normal);
                let reflected = reflected.unit_vector() + fuzz * Vec3::random_unit_vector();
                let scattered = Ray::new(rec.p, reflected, r_in.time());
                if scattered.direction().dot(&rec.normal) > 0.0 {
                    Some((scattered, *albedo))
                } else {
                    None
                }
            }
            Self::Dielectric { refraction_index } => {
                let attenuation = Color::new(1.0, 1.0, 1.0);
                let ri = if rec.front_face {
                    1.0 / refraction_index
                } else {
                    *refraction_index
                };

                let unit_d = r_in.direction().unit_vector();
                let cos_theta = (-unit_d).dot(&rec.normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let reflectance = {
                    let r0 = (1.0 - ri) / (1.0 + ri);
                    let r0 = r0 * r0;
                    r0 + (1.0 - r0) * (1.0 - cos_theta).powf(5.0)
                };

                let direction = if ri * sin_theta > 1.0 || reflectance > random_double() {
                    unit_d.reflect(&rec.normal)
                } else {
                    unit_d.refract(&rec.normal, ri)
                };

                Some((Ray::new(rec.p, direction, r_in.time()), attenuation))
            }
            _ => None,
        }
    }

    pub fn emitted(&self, u: f64, v: f64, p: Point3) -> Color {
        match self {
            Self::DiffuseLight { tex } => tex.value(u, v, &p),
            _ => Color::default(),
        }
    }
}

unsafe impl Send for Material {}
unsafe impl Sync for Material {}
