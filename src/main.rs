// use std::rc::Rc;
use std::sync::Arc;

use raytracing::bvh::BVHNode;
use raytracing::camera::Camera;
use raytracing::color::Color;
use raytracing::material::Material;
use raytracing::primitive::{build_box, ConstantMedium, HittableList, Planar, RotateY, Shape, Sphere, Translate};
use raytracing::texture::{CheckerTexture, NoiseTexture, SolidColor};
use raytracing::utils::{random_double, random_range};
use raytracing::vec3::{Point3, Vec3};

fn bouncing_spheres() {
    // World
    let mut world = HittableList::default();

    let checker = Arc::new(CheckerTexture::from((
        0.32,
        &Color::new(0.2, 0.3, 0.1),
        &Color::new(0.9, 0.9, 0.9),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::new(Material::Lambertian { tex: checker }),
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point3::new(
                a as f64 + 0.9 * random_double(),
                0.2,
                b as f64 + 0.9 * random_double(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() <= 0.9 {
                continue;
            }

            if choose_mat < 0.8 {
                // diffuse
                let albedo = Color::random() * Color::random();
                let sphere_material = Arc::new(Material::Lambertian {
                    tex: Arc::new(SolidColor::new(&albedo)),
                });
                let center2 = center + Vec3::new(0.0, random_range(0.0, 0.5), 0.0);
                world.add(Arc::new(Sphere::new_moving(
                    center,
                    center2,
                    0.2,
                    sphere_material,
                )));
            } else if choose_mat < 0.95 {
                // metal
                let albedo = Color::random_range(0.5, 1.0);
                let fuzz = random_range(0.0, 0.5);
                let sphere_material = Arc::new(Material::Metal { albedo, fuzz });
                world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
            } else {
                // glass
                let sphere_material = Arc::new(Material::Dielectric {
                    refraction_index: 1.5,
                });
                world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
            }
        }
    }

    let material1 = Arc::new(Material::Dielectric {
        refraction_index: 1.5,
    });
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1,
    )));

    let material2 = Arc::new(Material::Lambertian {
        tex: Arc::new(SolidColor::new(&Color::new(0.4, 0.2, 0.1))),
    });
    world.add(Arc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2,
    )));

    let material3 = Arc::new(Material::Metal {
        albedo: Color::new(0.7, 0.6, 0.5),
        fuzz: 0.0,
    });
    world.add(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3,
    )));

    // BVH seems to be slow... need to investigate
    let world = Arc::new(BVHNode::from(world));
    let world = Arc::new(HittableList::new(world));
    // let world = Arc::new(world);

    let mut cam = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 1200,
        samples_per_pixel: 512,
        max_depth: 50,
        background: Color::new(0.70, 0.80, 1.00),

        vfov: 20.0,
        lookfrom: Point3::new(13.0, 2.0, 3.0),
        lookat: Point3::new(0.0, 0.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.6,
        focus_dist: 10.0,

        ..Camera::default()
    };

    cam.render(world)
}

fn checkered_spheres() {
    let mut world = HittableList::default();

    let checker = Arc::new(CheckerTexture::from((
        0.32,
        &Color::new(0.2, 0.3, 0.1),
        &Color::new(0.9, 0.9, 0.9),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -10.0, 0.0),
        10.0,
        Arc::new(Material::Lambertian {
            tex: checker.clone(),
        }),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 10.0, 0.0),
        10.0,
        Arc::new(Material::Lambertian {
            tex: checker.clone(),
        }),
    )));

    let mut cam = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,
        background: Color::new(0.70, 0.80, 1.00),

        vfov: 20.0,
        lookfrom: Point3::new(13.0, 2.0, 3.0),
        lookat: Point3::new(0.0, 0.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,
        ..Camera::default()
    };

    cam.render(Arc::new(world))
}

fn perlin_spheres() {
    let mut world = HittableList::default();

    let pertext = Arc::new(NoiseTexture::new(4.));

    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::new(Material::Lambertian {
            tex: pertext.clone(),
        }),
    )));

    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::new(Material::Lambertian {
            tex: pertext.clone(),
        }),
    )));

    let mut cam = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,
        background: Color::new(0.70, 0.80, 1.00),

        vfov: 20.0,
        lookfrom: Point3::new(13.0, 2.0, 3.0),
        lookat: Point3::new(0.0, 0.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,
        ..Camera::default()
    };

    cam.render(Arc::new(world))
}

fn quads() {
    let mut world = HittableList::default();

    // Materials
    let left_red = Arc::new(Material::Lambertian {
        tex: Arc::new(SolidColor::new(&Color::new(1.0, 0.2, 0.2))),
    });
    let back_green = Arc::new(Material::Lambertian {
        tex: Arc::new(SolidColor::new(&Color::new(0.2, 1.0, 0.2))),
    });
    let right_blue = Arc::new(Material::Lambertian {
        tex: Arc::new(SolidColor::new(&Color::new(0.2, 0.2, 1.0))),
    });
    let upper_orange = Arc::new(Material::Lambertian {
        tex: Arc::new(SolidColor::new(&Color::new(1.0, 0.5, 0.0))),
    });
    let lower_teal = Arc::new(Material::Lambertian {
        tex: Arc::new(SolidColor::new(&Color::new(0.2, 0.8, 0.8))),
    });

    // Quads
    world.add(Arc::new(Planar::new(
        Point3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        left_red,
        Shape::Quad,
    )));
    world.add(Arc::new(Planar::new(
        Point3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        back_green,
        Shape::Quad,
    )));
    world.add(Arc::new(Planar::new(
        Point3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        right_blue,
        Shape::Quad,
    )));
    world.add(Arc::new(Planar::new(
        Point3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        upper_orange,
        Shape::Quad,
    )));
    world.add(Arc::new(Planar::new(
        Point3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        lower_teal,
        Shape::Quad,
    )));

    let mut cam = Camera {
        aspect_ratio: 1.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,
        background: Color::new(0.70, 0.80, 1.00),

        vfov: 80.0,
        lookfrom: Point3::new(0.0, 0.0, 9.0),
        lookat: Point3::new(0.0, 0.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,
        ..Camera::default()
    };

    cam.render(Arc::new(world));
}

fn simple_light() {
    let mut world = HittableList::default();

    let pertext = Arc::new(NoiseTexture::new(4.0));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::new(Material::Lambertian {
            tex: pertext.clone(),
        }),
    )));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::new(Material::Lambertian {
            tex: pertext.clone(),
        }),
    )));

    let difflight = Arc::new(Material::DiffuseLight {
        tex: Arc::new(SolidColor::new(&Color::new(4.0, 4.0, 4.0))),
    });
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        difflight.clone(),
    )));
    world.add(Arc::new(Planar::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight.clone(),
        Shape::Quad,
    )));

    let mut cam = Camera {
        aspect_ratio: 16.0 / 9.0,
        image_width: 400,
        samples_per_pixel: 100,
        max_depth: 50,
        background: Color::default(),

        vfov: 20.0,
        lookfrom: Point3::new(26.0, 3.0, 6.0),
        lookat: Point3::new(0.0, 2.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,
        ..Camera::default()
    };

    cam.render(Arc::new(world));
}

fn cornell_box() {
    let mut world = HittableList::default();

    let red = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.65, 0.05, 0.05))) });
    let white = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.73, 0.73, 0.73))) });
    let green = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.12, 0.45, 0.15))) });
    let light = Arc::new(Material::DiffuseLight { tex: Arc::new(SolidColor::new(&Color::new(10., 10., 10.))) });

    world.add(Arc::new(Planar::new(Point3::new(555., 0.0, 0.0), Vec3::new(0.0, 555., 0.0), Vec3::new(0.0, 0.0, 555.), green, Shape::Quad)));
    world.add(Arc::new(Planar::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 555., 0.0), Vec3::new(0.0, 0.0, 555.), red, Shape::Quad)));
    // world.add(Arc::new(Planar::new(Point3::new(343., 554., 332.), Vec3::new(-130., 0.0, 0.0), Vec3::new(0.0, 0.0, -105.), light, Shape::Quad)));
    world.add(Arc::new(Sphere::new(Point3::new(343., 580., 350.), 100.0, light)));
    world.add(Arc::new(Planar::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(555., 0.0, 0.0), Vec3::new(0.0, 0.0, 555.), white.clone(), Shape::Quad)));
    world.add(Arc::new(Planar::new(Point3::new(555., 555., 555.), Vec3::new(-555., 0.0, 0.0), Vec3::new(0.0, 0.0, -555.), white.clone(), Shape::Quad)));
    world.add(Arc::new(Planar::new(Point3::new(0.0, 0.0, 555.), Vec3::new(555., 0.0, 0.0), Vec3::new(0.0, 555., 0.0), white.clone(), Shape::Quad)));

    let box1 = build_box(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), white.clone());
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box1);

    let box2 = build_box(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 165.0, 165.0), white.clone());
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Arc::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
    world.add(box2);

    let mut cam = Camera {
        aspect_ratio: 1.0,
        image_width: 600,
        samples_per_pixel: 200,
        max_depth: 50,
        background: Color::default(),

        vfov: 40.,
        lookfrom: Point3::new(278., 278., -800.),
        lookat: Point3::new(278., 278., 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,
        ..Camera::default()
    };

    cam.render(Arc::new(world));
}

fn cornell_smoke() {
    let mut world = HittableList::default();

    let red = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.65, 0.05, 0.05))) });
    let white: Arc<Material> = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.73, 0.73, 0.73))) });
    let green = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.12, 0.45, 0.15))) });
    let light = Arc::new(Material::DiffuseLight { tex: Arc::new(SolidColor::new(&Color::new(7.0, 7.0, 7.0))) });

    world.add(Arc::new(Planar::new(Point3::new(555., 0.0, 0.0), Vec3::new(0.0, 555., 0.0), Vec3::new(0.0, 0.0, 555.), green, Shape::Quad)));
    world.add(Arc::new(Planar::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 555., 0.0), Vec3::new(0.0, 0.0, 555.), red, Shape::Quad)));
    world.add(Arc::new(Planar::new(Point3::new(113., 554., 127.), Vec3::new(330., 0.0, 0.0), Vec3::new(0.0, 0.0, 305.), light, Shape::Quad)));
    world.add(Arc::new(Planar::new(Point3::new(0.0, 555., 0.0), Vec3::new(555., 0.0, 0.0), Vec3::new(0.0, 0.0, 555.), white.clone(), Shape::Quad)));
    world.add(Arc::new(Planar::new(Point3::new(0.0, 0.0, 0.0), Vec3::new(555., 0.0, 0.0), Vec3::new(0.0, 0.0, 555.), white.clone(), Shape::Quad)));
    world.add(Arc::new(Planar::new(Point3::new(0.0, 0.0, 555.), Vec3::new(555., 0.0, 0.0), Vec3::new(0.0, 555., 0.0), white.clone(), Shape::Quad)));

    let box1 = build_box(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0, 330.0, 165.0), white.clone());
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));

    let box2 = build_box(Point3::new(0.0, 0.0, 0.0), Point3::new(165.0,165.0, 165.0), white.clone());
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Arc::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));    

    world.add(Arc::new(ConstantMedium::new(box1, 0.01, Arc::new(SolidColor::new(&Color::new(0.0, 0.0, 0.0))))));
    world.add(Arc::new(ConstantMedium::new(box2, 0.01, Arc::new(SolidColor::new(&Color::new(1.0, 1.0, 1.0))))));

    let mut cam = Camera {
        aspect_ratio: 1.0,
        image_width: 600,
        samples_per_pixel: 200,
        max_depth: 50,
        background: Color::default(),

        vfov: 40.0,
        lookfrom: Point3::new(278.0, 278.0, -800.0),
        lookat: Point3::new(278.0, 278.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,
        ..Camera::default()
    };

    cam.render(Arc::new(world));
}

fn final_scene(image_width: i32, samples_per_pixel: i32, max_depth: i32) {
    let mut boxes1 = HittableList::default();
    let ground = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.48, 0.83, 0.53))) });

    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = random_range(1.0, 101.0);
            let z1 = z0 + w;

            boxes1.add(build_box(Point3::new(x0, y0, z0), Point3::new(x1, y1, z1), ground.clone()));
        }
    }

    let mut world = HittableList::default();

    world.add(Arc::new(BVHNode::from(boxes1)));

    let light = Arc::new(Material::DiffuseLight { tex: Arc::new(SolidColor::new(&Color::new(7.0, 7.0, 7.0))) });
    world.add(Arc::new(Planar::new(Point3::new(123.0, 554.0, 147.0), Vec3::new(300.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 265.0), light.clone(), Shape::Quad)));

    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_material = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.7, 0.3, 0.1))) });
    world.add(Arc::new(Sphere::new_moving(center1, center2, 50.0, sphere_material.clone())));

    world.add(Arc::new(Sphere::new(Point3::new(260.0, 150.0, 45.0), 50.0, Arc::new(Material::Dielectric { refraction_index: 1.5 }))));
    world.add(Arc::new(Sphere::new(Point3::new(0.0, 150.0, 145.0), 50.0, Arc::new(Material::Metal { albedo: Color::new(0.8, 0.8, 0.9), fuzz: 1.0 }))));

    let boundary = Arc::new(Sphere::new(Point3::new(360.0, 150.0, 145.0), 70.0, Arc::new(Material::Dielectric { refraction_index: 1.5 })));
    world.add(boundary.clone());
    world.add(Arc::new(ConstantMedium::new(boundary.clone(), 0.2, Arc::new(SolidColor::new(&Color::new(0.2, 0.4, 0.9))))));
    let boundary = Arc::new(Sphere::new(Point3::new(0.0, 0.0, 0.0), 5000.0, Arc::new(Material::Dielectric { refraction_index: 1.5 })));
    world.add(Arc::new(ConstantMedium::new(boundary.clone(), 0.0001, Arc::new(SolidColor::new(&Color::new(1.0, 1.0, 1.0))))));

    let pertext = Arc::new(NoiseTexture::new(0.2));
    world.add(Arc::new(Sphere::new(Point3::new(220.0, 280.0, 300.0), 80.0, Arc::new(Material::Lambertian { tex: pertext.clone() }))));

    let mut boxes2 = HittableList::default();
    let white = Arc::new(Material::Lambertian { tex: Arc::new(SolidColor::new(&Color::new(0.73, 0.73, 0.73))) });
    let ns = 1000;
    for _ in 0..ns {
        boxes2.add(Arc::new(Sphere::new(Point3::random_range(0.0, 165.0), 10.0, white.clone())));
    }

    world.add(Arc::new(Translate::new(Arc::new(RotateY::new(Arc::new(BVHNode::from(boxes2)), 15.0)), Vec3::new(-100.0, 270.0, 395.0))));

    let mut cam = Camera {
        aspect_ratio: 1.0,
        image_width,
        samples_per_pixel,
        max_depth,
        background: Color::default(),

        vfov: 40.0,
        lookfrom: Point3::new(478.0, 278.0, -600.0),
        lookat: Point3::new(278.0, 278.0, 0.0),
        vup: Vec3::new(0.0, 1.0, 0.0),

        defocus_angle: 0.0,

        ..Camera::default()
    };

    cam.render(Arc::new(world));
}
fn main() {
    let mut scene = String::new();
    eprintln!("Input scene index: ");
    eprintln!("-- 0. Bouncing Spheres");
    eprintln!("-- 1. Checkered Spheres");
    eprintln!("-- 2. Perlin Spheres");
    eprintln!("-- 3. Planar");
    eprintln!("-- 4. Simple Light");
    eprintln!("-- 5. Cornell Box");
    eprintln!("-- 6. Cornell Smoke");
    eprintln!("-- 7. Final Scene Test");
    eprintln!("-- 8. Final Scene Release");
    std::io::stdin()
        .read_line(&mut scene)
        .expect("Invalid input");
    scene.pop();
    match scene.parse::<i32>() {
        Ok(0) => bouncing_spheres(),
        Ok(1) => checkered_spheres(),
        Ok(2) => perlin_spheres(),
        Ok(3) => quads(),
        Ok(4) => simple_light(),
        Ok(5) => cornell_box(),
        Ok(6) => cornell_smoke(),
        Ok(7) => final_scene(400, 250, 4),
        Ok(8) => final_scene(800, 10000, 40),
        _ => {
            eprintln!("Invalid Scene index: {scene}");
        }
    }
}
