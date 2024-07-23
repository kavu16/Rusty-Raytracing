use std::{cmp::Ordering, sync::Arc};

use crate::{
    aabb::{AABB, EMPTY},
    interval::Interval,
    primitive::{HitRecord, Hittable, HittableList},
    ray::Ray,
};

#[derive(Clone)]
pub struct BVHNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: AABB,
}

impl BVHNode {
    pub fn new(objects: &mut Vec<Arc<dyn Hittable>>, start: usize, end: usize) -> Self {
        let mut bbox = EMPTY;
        for object in objects.iter_mut() {
            bbox = AABB::from((bbox, object.bounding_box()));
        }

        let axis = bbox.longest_axis();
        let comparator = if axis == 0 {
            BVHNode::box_x_compare
        } else if axis == 1 {
            BVHNode::box_y_compare
        } else {
            BVHNode::box_z_compare
        };

        let object_span = end - start;
        match object_span {
            1 => {
                let (left, right) = (objects[start].clone(), objects[start].clone());
                Self {
                    left: left.clone(),
                    right: right.clone(),
                    bbox,
                }
            }
            2 => {
                let left = &objects[start];
                let right = &objects[start + 1];
                Self {
                    left: left.clone(),
                    right: right.clone(),
                    bbox,
                }
            }
            _ => {
                objects[start..end].sort_by(|a, b| comparator(a.clone(), b.clone()));

                let mid = start + object_span / 2;
                let left = Arc::new(Self::new(objects, start, mid));
                let right = Arc::new(Self::new(objects, mid, end));
                Self {
                    left: left.clone(),
                    right: right.clone(),
                    bbox,
                }
            }
        }
    }

    fn box_compare(a: Arc<dyn Hittable>, b: Arc<dyn Hittable>, axis_index: i32) -> Ordering {
        let a_axis_interval = a.bounding_box().axis_interval(axis_index);
        let b_axis_interval = b.bounding_box().axis_interval(axis_index);
        a_axis_interval.min.total_cmp(&b_axis_interval.min)
    }

    fn box_x_compare(a: Arc<dyn Hittable>, b: Arc<dyn Hittable>) -> Ordering {
        BVHNode::box_compare(a, b, 0)
    }

    fn box_y_compare(a: Arc<dyn Hittable>, b: Arc<dyn Hittable>) -> Ordering {
        BVHNode::box_compare(a, b, 1)
    }

    fn box_z_compare(a: Arc<dyn Hittable>, b: Arc<dyn Hittable>) -> Ordering {
        BVHNode::box_compare(a, b, 2)
    }
}

impl Hittable for BVHNode {
    fn hit(&self, r: &Ray, ray_t: &mut Interval) -> Option<HitRecord> {
        if !self.bbox.hit(r, ray_t) {
            return None;
        }

        let left_hit = self.left.hit(r, ray_t);
        if let Some(rec) = &left_hit {
            self.right
                .hit(r, &mut Interval::new(ray_t.min, rec.t))
                .or(left_hit)
        } else {
            self.right.hit(r, ray_t)
        }
    }

    fn bounding_box(&self) -> AABB {
        self.bbox
    }
}

impl From<HittableList> for BVHNode {
    fn from(list: HittableList) -> Self {
        Self::new(&mut list.clone().objects, 0, list.objects.len())
    }
}

unsafe impl Send for BVHNode {}
unsafe impl Sync for BVHNode {}
