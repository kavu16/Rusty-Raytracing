use std::sync::Arc;

// use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::prelude::*;

use crate::{
    color::Color,
    interval::Interval,
    primitive::{Hittable, HittableList},
    ray::Ray,
    utils::{degrees_to_radians, random_double},
    vec3::{Point3, Vec3},
};

#[derive(Clone, Copy)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,
    pub background: Color,

    pub vfov: f64,
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,

    pub defocus_angle: f64,
    pub focus_dist: f64,

    pub image_height: i32,
    pub pixel_samples_scale: f64,
    pub center: Point3,
    pub pixel00_loc: Point3,
    pub pixel_delta_u: Vec3,
    pub pixel_delta_v: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
    pub defocus_disk_u: Vec3,
    pub defocus_disk_v: Vec3,
}

impl Camera {
    fn ray_color(&self, r: Ray, depth: i32, world: Arc<dyn Hittable>) -> Color {
        if depth <= 0 {
            return Color::default();
        }
        if let Some(rec) = world.hit(&r, &mut Interval::new(0.001, f64::INFINITY)) {
            let color_from_emission = rec.mat.emitted(rec.u, rec.v, rec.p);
            if let Some((scattered, attenuation)) = rec.mat.scatter(r, &rec) {
                let color_from_scatter =
                    attenuation * Camera::ray_color(self, scattered, depth - 1, world);
                return color_from_emission + color_from_scatter;
            }
            return color_from_emission;
        }
        self.background
    }

    fn initialize(&mut self) {
        self.image_height = ((self.image_width as f64 / self.aspect_ratio) as i32).max(1);

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.lookfrom;

        // Camera viewport
        let theta = degrees_to_radians(self.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // Calculate u,v,w unit basis vecs
        self.w = (self.lookfrom - self.lookat).unit_vector();
        self.u = self.vup.cross(&self.w).unit_vector();
        self.v = self.w.cross(&self.u);

        // vertical and horizontal edges
        let viewport_u = viewport_width * self.u;
        let viewport_v = viewport_height * -self.v;

        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        let viewport_upper_left =
            self.center - (self.focus_dist * self.w) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        let defocus_radius = self.focus_dist * (degrees_to_radians(self.defocus_angle / 2.0)).tan();
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    fn sample_square() -> Vec3 {
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p = Vec3::random_in_unit_disk();
        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    fn get_ray(&self, i: i32, j: i32) -> Ray {
        let offset = Camera::sample_square();
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;
        let ray_time = random_double();

        Ray::new(ray_origin, ray_direction, ray_time)
    }

    pub fn render(&mut self, world: Arc<HittableList>) {
        self.initialize();

        // Render
        println!("P3\n{} {}\n255\n", self.image_width, self.image_height);
        for j in 0..self.image_height {
            eprint!("\rScanlines remaining: {}    ", self.image_height - j);
            for i in 0..self.image_width {
                let pixel_color: Color = (0..self.samples_per_pixel)
                    .into_par_iter()
                    .map_init(
                        || self.get_ray(i, j),
                        |r, _s| Camera::ray_color(self, *r, self.max_depth, world.clone()),
                    )
                    .sum();
                Color::write_color(self.pixel_samples_scale * pixel_color);
            }
        }
        eprint!("\rDone.                    \n");
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            background: Color::default(),

            vfov: 90.0,
            lookfrom: Point3::new(0.0, 0.0, 0.0),
            lookat: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),

            defocus_angle: 0.0,
            focus_dist: 10.0,

            image_height: i32::default(),
            pixel_samples_scale: f64::default(),
            center: Point3::default(),
            pixel00_loc: Point3::default(),
            pixel_delta_u: Vec3::default(),
            pixel_delta_v: Vec3::default(),
            u: Vec3::default(),
            v: Vec3::default(),
            w: Vec3::default(),
            defocus_disk_u: Vec3::default(),
            defocus_disk_v: Vec3::default(),
        }
    }
}
