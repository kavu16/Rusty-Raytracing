use crate::{color::Color, primitive::HitRecord, ray::Ray, vec3::Vec3};

#[derive(Clone, Copy, Debug)]
pub enum Material {
    Lambertian { albedo: Color },
    Metal { albedo: Color, fuzz: f64 },
    Dielectric { refraction_index: f64 }
}

impl Material {
    pub fn scatter(&self, r_in: Ray, rec: &HitRecord) -> Option<(Ray, Color)> {
        match self {
            Self::Lambertian { albedo } => {
                let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

                if scatter_direction.near_zero() {
                    scatter_direction = rec.normal;
                }

                Some((Ray::new(&rec.p, &scatter_direction), *albedo))
            }
            Self::Metal { albedo, fuzz } => {
                let fuzz = fuzz.min(1.0);
                let reflected = r_in.direction().reflect(&rec.normal);
                let reflected = reflected.unit_vector() + fuzz * Vec3::random_unit_vector();
                let scattered = Ray::new(&rec.p, &reflected);
                if scattered.direction().dot(&rec.normal) > 0.0 {
                    Some((scattered, *albedo))
                } else {
                    None
                }
            }
            Self::Dielectric { refraction_index } => {
                let attenuation = Color::new(1.0, 1.0, 1.0);
                let ri = if rec.front_face { 1.0 / refraction_index } else { *refraction_index };

                let unit_direction = Vec3::unit_vector(&r_in.direction());
                let refracted = unit_direction.refract(&rec.normal, ri);

                Some((Ray::new(&rec.p, &refracted), attenuation))
            }
        }
    }
}
