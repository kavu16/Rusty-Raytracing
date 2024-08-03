use std::f64::consts::PI;
// use std::rc::Rc;
use std::sync::Arc;

use crate::aabb::AABB;
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
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

pub trait Hittable {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord>;
    fn bounding_box(&self) -> AABB;
}

pub struct Sphere {
    center1: Point3,
    radius: f64,
    mat: Arc<Material>,
    is_moving: bool,
    center_vec: Vec3,
    bbox: AABB,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Arc<Material>) -> Self {
        Self {
            center1: center,
            radius: radius.max(0.0),
            mat,
            is_moving: false,
            center_vec: Vec3::default(),
            bbox: {
                let rvec = Vec3::new(radius, radius, radius);
                AABB::from((center - rvec, center + rvec))
            },
        }
    }

    pub fn new_moving(center1: Point3, center2: Point3, radius: f64, mat: Arc<Material>) -> Self {
        Self {
            center1,
            radius: radius.max(0.0),
            mat,
            is_moving: true,
            center_vec: center2 - center1,
            bbox: {
                let rvec = Vec3::new(radius, radius, radius);
                let box1 = AABB::from((center1 - rvec, center1 + rvec));
                let box2 = AABB::from((center2 - rvec, center2 + rvec));
                AABB::from((box1, box2))
            },
        }
    }

    fn sphere_center(&self, time: f64) -> Point3 {
        self.center1 + time * self.center_vec
    }

    fn get_sphere_uv(&self, p: Point3) -> (f64, f64) {
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;

        (phi / (2.0*PI), theta / PI)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord> {
        let center = if self.is_moving {
            self.sphere_center(r.time())
        } else {
            self.center1
        };
        let oc = center - r.origin();
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
        let normal = (p - center) / self.radius;
        let front_face = r.direction().dot(&normal) < 0.0;
        let normal = if front_face { normal } else { -normal };
        let mat = self.mat.clone();
        let (u, v) = self.get_sphere_uv(normal);

        Some(HitRecord {
            t,
            p,
            normal,
            mat,
            front_face,
            u,
            v,
        })
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}
unsafe impl Send for Sphere {}
unsafe impl Sync for Sphere {}

#[derive(Clone, Default)]
pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable>>,
    bbox: AABB,
}

impl HittableList {
    pub fn new(object: Arc<dyn Hittable>) -> Self {
        let mut hit_list = HittableList::default();
        hit_list.add(object);
        hit_list
    }

    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        self.bbox = AABB::from((self.bbox, object.bounding_box()));
        self.objects.push(object);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord> {
        self.objects.iter().fold((ray_t.max, None), |(closest, curr_rec), object| {
            if let Some(temp_rec) = object.hit(r, &mut Interval::new(ray_t.min, closest)) {
                (temp_rec.t, Some(temp_rec))
            } else {
                (closest, curr_rec)
            }
        }).1 // Returning curr_rec
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}

unsafe impl Send for HittableList {}
unsafe impl Sync for HittableList {}

pub enum Shape {
    Quad,
    Triangle,
    Circle { radius: f64 },
}

unsafe impl Send for Shape {}
unsafe impl Sync for Shape {}

pub struct Planar {
    q: Point3,
    u: Vec3, v: Vec3,
    w: Vec3,
    mat: Arc<Material>,
    bbox: AABB,
    normal: Vec3,
    d: f64,
    shape: Shape,
}

impl Planar {
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Arc<Material>, shape: Shape) -> Self {
        let n = u.cross(&v);
        let normal = n.unit_vector();
        let d = normal.dot(&q);
        let w = n / n.dot(&n);
        let mut planar = Self {
            q,
            u,
            v,
            w,
            mat,
            bbox: AABB::default(),
            normal,
            d,
            shape,
        };
        planar.set_bounding_box();
        planar
    }

    pub fn set_bounding_box(&mut self) {
        let bbox_diagonal1 = AABB::from((self.q, self.q + self.u + self.v));
        let bbox_diagonal2 = AABB::from((self.q + self.u, self.q + self.v));
        self.bbox = AABB::from((bbox_diagonal1, bbox_diagonal2));
    }
}

impl Hittable for Planar {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord> {
        let denom = self.normal.dot(&r.direction());

        // no hit if parallel
        if denom.abs() < 1e-8 { return None; }

        // no hit if t outside ray interval
        let t = (self.d - self.normal.dot(&r.origin())) / denom;
        if !ray_t.contains(t) { return None; }

        // Determine if hit lies within quad
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let alpha = self.w.dot(&planar_hitpt_vector.cross(&self.v));
        let beta = self.w.dot(&self.u.cross(&planar_hitpt_vector));

        match self.shape {
            Shape::Quad => {
                let unit_interval = Interval::new(0.0, 1.0);
                if !unit_interval.contains(alpha) || !unit_interval.contains(beta) {
                    return None;
                }
            }
            Shape::Circle { radius } => {
                if (alpha * alpha + beta * beta).sqrt() > radius {
                    return None;
                }
            }
            Shape::Triangle => {
                if !(alpha > 0.0 && beta > 0.0 && alpha + beta < 1.0) {
                    return None
                }
            }
        }

        let front_face = r.direction().dot(&self.normal) < 0.0;
        let normal = if front_face { self.normal } else { -self.normal };

        Some(HitRecord {
            t, 
            p: intersection,
            normal,
            mat: self.mat.clone(),
            front_face,
            u: alpha,
            v: beta,
        })
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}

unsafe impl Send for Planar {}
unsafe impl Sync for Planar {}