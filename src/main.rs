use std::rc::Rc;

use raytracing::camera::Camera;
use raytracing::color::Color;
use raytracing::material::Material;
use raytracing::primitive::{HittableList, Sphere};
use raytracing::utils::{random_double, random_range};
use raytracing::vec3::{Point3, Vec3};

fn main() {
    // World
    let mut world = HittableList::default();

    let ground_material = Rc::new(Material::Lambertian { 
        albedo: Color::new(0.5, 0.5, 0.5)
     });
    world.add(Rc::new(Sphere::new(
        &Point3::new(0.0, -1000.0, 0.0),
        1000.0, 
        ground_material,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point3::new(
                a as f64 + 0.9 * random_double(), 
                0.2, 
                b as f64 + 0.9 * random_double(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() <= 0.9 { continue; }

            if choose_mat < 0.8 {
                // diffuse
                let albedo = Color::random() * Color::random();
                let sphere_material = Rc::new(Material::Lambertian { albedo });
                world.add(Rc::new(Sphere::new(&center, 0.2, sphere_material)));
            } else if choose_mat < 0.95 {
                // metal
                let albedo = Color::random_range(0.5, 1.0);
                let fuzz = random_range(0.0, 0.5);
                let sphere_material = Rc::new(Material::Metal { albedo, fuzz });
                world.add(Rc::new(Sphere::new(&center, 0.2, sphere_material)));
            } else {
                // glass
                let sphere_material = Rc::new(Material::Dielectric { refraction_index: 1.5 });
                world.add(Rc::new(Sphere::new(&center, 0.2, sphere_material)));
            }
        }
    }

    let material1 = Rc::new(Material::Dielectric { refraction_index: 1.5 });
    world.add(Rc::new(Sphere::new(&Point3::new(0.0, 1.0, 0.0), 1.0, material1)));

    let material2 = Rc::new(Material::Lambertian { albedo: Color::new(0.4, 0.2, 0.1) });
    world.add(Rc::new(Sphere::new(&Point3::new(-4.0, 1.0, 0.0), 1.0, material2)));

    let material3 = Rc::new(Material::Metal { albedo: Color::new(0.7, 0.6, 0.5), fuzz: 0.0 });
    world.add(Rc::new(Sphere::new(&Point3::new(4.0, 1.0, 0.0), 1.0, material3)));
    

    let mut cam = Camera::default();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 10;
    cam.max_depth = 50;

    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(13.0, 2.0, 3.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.6;
    cam.focus_dist = 10.0;

    cam.render(&world)
}
