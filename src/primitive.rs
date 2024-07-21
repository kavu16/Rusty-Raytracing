// use std::rc::Rc;
use std::sync::Arc;

use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::*;

#[derive(Clone)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Arc<Material>,
    pub t: f64,
    pub front_face: bool,
}

pub trait Hittable {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord>;
}

pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<Material>,
}

impl Sphere {
    pub fn new(center: &Point3, radius: f64, mat: Arc<Material>) -> Self {
        Self {
            center: *center,
            radius: radius.max(0.0),
            mat,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let oc = self.center - r.origin();
        let a = r.direction().length_squared();
        let h = r.direction().dot(&oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return None;
            }
        }
        let t = root;
        let p = r.at(t);
        let normal = (p - self.center).unit_vector();
        let front_face = r.direction().dot(&normal) < 0.0;
        let normal = if front_face { normal } else { -normal };
        let mat = self.mat.clone();

        Some(HitRecord {
            t,
            p,
            normal,
            mat,
            front_face,
        })
    }
}
unsafe impl Send for Sphere {}
unsafe impl Sync for Sphere {}

#[derive(Clone, Default)]
pub struct HittableList {
    objects: Vec<Arc<dyn Hittable>>,
}

impl HittableList {
    pub fn new(object: Arc<dyn Hittable>) -> Self {
        Self {
            objects: vec![object],
        }
    }

    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        self.objects.push(object);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let mut rec = None;
        let mut closest = ray_t.max;

        for object in self.objects.iter() {
            if let Some(temp_rec) = object.hit(r, Interval::new(ray_t.min, closest)) {
                closest = temp_rec.t;
                rec = Some(temp_rec);
            }
        }

        rec
    }
}

unsafe impl Send for HittableList {}
unsafe impl Sync for HittableList {}
