use std::f64::consts::PI;
use std::fmt::Debug;
// use std::rc::Rc;
use std::sync::Arc;

use crate::aabb::AABB;
use crate::interval::{Interval, UNIVERSE};
use crate::material::Material;
use crate::ray::Ray;
use crate::texture::Texture;
use crate::utils::{degrees_to_radians, random_double};
use crate::vec3::*;

#[derive(Clone, Debug)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Arc<Material>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

pub trait Hittable: Debug {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord>;
    fn bounding_box(&self) -> AABB;
}

#[derive(Clone, Debug)]
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

        (phi / (2.0 * PI), theta / PI)
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

#[derive(Clone, Default, Debug)]
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
        self.bbox = AABB::default();
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord> {
        self.objects
            .iter()
            .fold((ray_t.max, None), |(closest, curr_rec), object| {
                if let Some(temp_rec) = object.hit(r, &mut Interval::new(ray_t.min, closest)) {
                    (temp_rec.t, Some(temp_rec))
                } else {
                    (closest, curr_rec)
                }
            })
            .1 // Returning curr_rec
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}

unsafe impl Send for HittableList {}
unsafe impl Sync for HittableList {}

#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Quad,
    Triangle,
    Circle { radius: f64 },
}

unsafe impl Send for Shape {}
unsafe impl Sync for Shape {}

#[derive(Debug, Clone)]
pub struct Planar {
    q: Point3,
    u: Vec3,
    v: Vec3,
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
        if denom.abs() < 1e-8 {
            return None;
        }

        // no hit if t outside ray interval
        let t = (self.d - self.normal.dot(&r.origin())) / denom;
        if !ray_t.contains(t) {
            return None;
        }

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
                    return None;
                }
            }
        }

        let front_face = r.direction().dot(&self.normal) < 0.0;
        let normal = if front_face {
            self.normal
        } else {
            -self.normal
        };

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

#[inline]
pub fn build_box(a: Point3, b: Point3, mat: Arc<Material>) -> Arc<HittableList> {
    let mut sides = HittableList::default();

    let min = Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

    let dx = Vec3::new(max.x - min.x, 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y - min.y, 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z - min.z);

    sides.add(Arc::new(Planar::new(
        Point3::new(min.x, min.y, max.z),
        dx,
        dy,
        mat.clone(),
        Shape::Quad,
    )));
    sides.add(Arc::new(Planar::new(
        Point3::new(max.x, min.y, max.z),
        -dz,
        dy,
        mat.clone(),
        Shape::Quad,
    )));
    sides.add(Arc::new(Planar::new(
        Point3::new(max.x, min.y, min.z),
        -dx,
        dy,
        mat.clone(),
        Shape::Quad,
    )));
    sides.add(Arc::new(Planar::new(
        Point3::new(min.x, min.y, min.z),
        dz,
        dy,
        mat.clone(),
        Shape::Quad,
    )));
    sides.add(Arc::new(Planar::new(
        Point3::new(min.x, max.y, max.z),
        dx,
        -dz,
        mat.clone(),
        Shape::Quad,
    )));
    sides.add(Arc::new(Planar::new(
        Point3::new(min.x, min.y, min.z),
        dx,
        dz,
        mat.clone(),
        Shape::Quad,
    )));

    Arc::new(sides)
}

#[derive(Debug, Clone)]
pub struct Translate {
    object: Arc<dyn Hittable>,
    offset: Vec3,
    bbox: AABB,
}

impl Translate {
    pub fn new(object: Arc<dyn Hittable>, offset: Vec3) -> Self {
        Self {
            object: object.clone(),
            offset,
            bbox: object.clone().bounding_box() + offset,
        }
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord> {
        let offset_r = Ray::new(r.origin() - self.offset, r.direction(), r.time());

        if let Some(rec) = self.object.hit(&offset_r, ray_t) {
            return Some(HitRecord {
                p: rec.p + self.offset,
                ..rec
            });
        }

        None
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}

unsafe impl Send for Translate {}
unsafe impl Sync for Translate {}

#[derive(Debug, Clone)]
pub struct RotateY {
    object: Arc<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: AABB,
}

impl RotateY {
    pub fn new(object: Arc<dyn Hittable>, angle: f64) -> Self {
        let radians = degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox = object.clone().bounding_box();

        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f64 * bbox.x.max + (1.0 - i as f64) * bbox.x.min;
                    let y = j as f64 * bbox.y.max + (1.0 - j as f64) * bbox.y.min;
                    let z = k as f64 * bbox.z.max + (1.0 - k as f64) * bbox.z.min;

                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;

                    let tester = Vec3::new(newx, y, newz);

                    for c in 0..3 {
                        min[c] = min[c].min(tester[c]);
                        max[c] = max[c].max(tester[c]);
                    }
                }
            }
        }

        Self {
            object: object.clone(),
            sin_theta,
            cos_theta,
            bbox: AABB::from((min, max)),
        }
    }
}

impl Hittable for RotateY {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord> {
        let mut origin = r.origin();
        let mut direction = r.direction();

        origin[0] = self.cos_theta * r.origin()[0] - self.sin_theta * r.origin()[2];
        origin[2] = self.sin_theta * r.origin()[0] + self.cos_theta * r.origin()[2];

        direction[0] = self.cos_theta * r.direction()[0] - self.sin_theta * r.direction()[2];
        direction[2] = self.sin_theta * r.direction()[0] + self.cos_theta * r.direction()[2];

        let rotated_r = Ray::new(origin, direction, r.time());

        if let Some(rec) = self.object.hit(&rotated_r, ray_t) {
            let mut p = rec.p;
            p[0] = self.cos_theta * rec.p[0] + self.sin_theta * rec.p[2];
            p[2] = -self.sin_theta * rec.p[0] + self.cos_theta * rec.p[2];

            let mut normal = rec.normal;
            normal[0] = self.cos_theta * rec.normal[0] + self.sin_theta * rec.normal[2];
            normal[2] = -self.sin_theta * rec.normal[0] + self.cos_theta * rec.normal[2];

            return Some(HitRecord { p, normal, ..rec });
        }

        None
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}

unsafe impl Send for RotateY {}
unsafe impl Sync for RotateY {}

#[derive(Debug, Clone)]
pub struct ConstantMedium {
    boundary: Arc<dyn Hittable>,
    neg_inv_density: f64,
    phase_function: Arc<Material>,
}

impl ConstantMedium {
    pub fn new(boundary: Arc<dyn Hittable>, density: f64, tex: Arc<dyn Texture>) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Arc::new(Material::Isotropic { tex }),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord> {
        if let Some(mut rec1) = self.boundary.hit(r, &mut UNIVERSE) {
            if let Some(mut rec2) = self
                .boundary
                .hit(r, &mut Interval::new(rec1.t + 0.0001, f64::INFINITY))
            {
                if rec1.t < ray_t.min {
                    rec1.t = ray_t.min;
                }
                if rec2.t > ray_t.max {
                    rec2.t = ray_t.max;
                }

                if rec1.t >= rec2.t {
                    return None;
                }

                if rec1.t < 0.0 {
                    rec1.t = 0.0;
                }

                let ray_length = r.direction().length();
                let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
                let hit_distance = self.neg_inv_density * random_double().ln();

                if hit_distance > distance_inside_boundary {
                    return None;
                }

                let t = rec1.t + hit_distance / ray_length;
                let p = r.at(t);
                let normal = Vec3::new(1.0, 0.0, 0.0);
                let front_face = true;
                let mat = self.phase_function.clone();

                return Some(HitRecord {
                    p,
                    normal,
                    mat,
                    t,
                    u: rec2.u,
                    v: rec2.v,
                    front_face,
                });
            }
        }

        None
    }

    fn bounding_box(&self) -> AABB {
        self.boundary.bounding_box()
    }
}

unsafe impl Send for ConstantMedium {}
unsafe impl Sync for ConstantMedium {}
