use crate::{
    interval::{Interval, EMPTY as IEmpty, UNIVERSE as IUniverse},
    ray::Ray,
    vec3::Point3,
};

#[derive(Clone, Copy, Default)]
pub struct AABB {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl AABB {
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn axis_interval(&self, n: i32) -> Interval {
        if n == 1 {
            self.y
        } else if n == 2 {
            self.z
        } else {
            self.x
        }
    }

    pub fn hit(&self, r: &Ray, mut ray_t: Interval) -> bool {
        let ray_orig = r.origin();
        let ray_dir = r.direction();

        for axis in 0..3 {
            let ax = self.axis_interval(axis);
            let adinv = 1.0 / ray_dir[axis];

            let t0 = (ax.min - ray_orig[axis]) * adinv;
            let t1 = (ax.max - ray_orig[axis]) * adinv;

            if t0 < t1 {
                if t0 > ray_t.min {
                    ray_t.min = t0;
                }
                if t1 < ray_t.max {
                    ray_t.max = t1;
                }
            } else {
                if t1 > ray_t.min {
                    ray_t.min = t1;
                }
                if t0 < ray_t.max {
                    ray_t.max = t0;
                }
            }

            if ray_t.max <= ray_t.min {
                return false;
            }
        }

        true
    }

    #[inline]
    pub fn longest_axis(&self) -> i32 {
        if self.x.size() > self.y.size() && self.x.size() > self.z.size() {
            0
        } else if self.y.size() > self.z.size() {
            1
        } else {
            2
        }
        
    }
}

impl From<(Point3, Point3)> for AABB {
    fn from((a, b): (Point3, Point3)) -> Self {
        Self {
            x: if a.x <= b.x {
                Interval::new(a.x, b.x)
            } else {
                Interval::new(b.x, a.x)
            },
            y: if a.y <= b.y {
                Interval::new(a.y, b.y)
            } else {
                Interval::new(b.y, a.y)
            },
            z: if a.z <= b.z {
                Interval::new(a.z, b.z)
            } else {
                Interval::new(b.z, a.z)
            },
        }
    }
}

impl From<(AABB, AABB)> for AABB {
    fn from((box0, box1): (AABB, AABB)) -> Self {
        Self {
            x: Interval::from((box0.x, box1.x)),
            y: Interval::from((box0.y, box1.y)),
            z: Interval::from((box0.z, box1.z)),
        }
    }
}

pub const EMPTY: AABB = AABB {
    x: IEmpty,
    y: IEmpty,
    z: IEmpty,
};
pub const UNIVERSE: AABB = AABB {
    x: IUniverse,
    y: IUniverse,
    z: IUniverse,
};
