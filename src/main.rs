mod vmath;

use std::fs::File;
use std::io::Write;

use vmath::{dot, normalize, ray, Ray, Vec3, vec3};

struct RayHit {
    t: f32,
    p: Vec3,
    normal: Vec3,
}

trait Hitable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit>;
}

struct Sphere {
    centre: Vec3,
    radius: f32,
}

fn sphere(centre: Vec3, radius: f32) -> Sphere {
    Sphere { centre, radius }
}

impl Hitable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let oc = ray.origin - self.centre;
        let a = dot(ray.direction, ray.direction);
        let b = dot(oc, ray.direction);
        let c = dot(oc, oc) - self.radius * self.radius;
        let discriminant = b * b - a * c;
        if discriminant > 0.0 {
            let discriminant_sqrt = discriminant.sqrt();
            let t = (-b - discriminant_sqrt) / a;
            if t < t_max && t > t_min {
                let p = ray.point_at_parameter(t);
                let normal = (p - self.centre) / self.radius;
                return Some(RayHit { t, p, normal });
            }
            let t = (-b + discriminant_sqrt) / a;
            if t < t_max && t > t_min {
                let p = ray.point_at_parameter(t);
                let normal = (p - self.centre) / self.radius;
                return Some(RayHit { t, p, normal });
            }
        }
        None
    }
}

fn ray_to_colour(ray: &Ray, scene: &Scene) -> Vec3 {
    if let Some(ray_hit) = scene.hit(ray, 0.0, std::f32::MAX) {
        0.5
            * vec3(
                ray_hit.normal.x + 1.0,
                ray_hit.normal.y + 1.0,
                ray_hit.normal.z + 1.0,
            )
    } else {
        let unit_direction = normalize(ray.direction);
        let t = 0.5 * (unit_direction.y + 1.0);
        (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0)
    }
}

// fn hit_sphere(centre: Vec3, radius: f32, ray: &Ray) -> f32 {
//     let oc = ray.origin - centre;
//     let a = dot(ray.direction, ray.direction);
//     let b = 2.0 * dot(oc, ray.direction);
//     let c = dot(oc, oc) - radius * radius;
//     let discriminant = b * b - 4.0 * a * c;
//     if discriminant < 0.0 {
//         -1.0
//     } else {
//         (-b - discriminant.sqrt()) / (2.0 * a)
//     }
// }

struct Scene {
    spheres: Vec<Sphere>,
}

impl Scene {
    fn new(spheres: Vec<Sphere>) -> Scene {
        Scene { spheres }
    }
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let mut result = None;
        let mut closest_so_far = t_max;
        for sphere in &self.spheres {
            if let Some(ray_hit) = sphere.hit(ray, t_min, closest_so_far) {
                closest_so_far = ray_hit.t;
                result = Some(ray_hit);
            }
        }
        result
    }
}

fn main() {
    let nx = 200;
    let ny = 100;
    let lower_left_corner = vec3(-2.0, -1.0, -1.0);
    let horizontal = vec3(4.0, 0.0, 0.0);
    let vertical = vec3(0.0, 2.0, 0.0);
    let origin = vec3(0.0, 0.0, 0.0);
    let scene = Scene::new(vec![
        sphere(vec3(0.0, 0.0, -1.0), 0.5),
        sphere(vec3(0.0, -100.5, -1.0), 100.0),
    ]);
    let mut out = File::create("output.ppm").expect("Failed to create output.ppm");
    write!(out, "P3\n{} {} \n255\n", nx, ny).expect("Failed to write to output.ppm");
    for j in 0..(ny - 1) {
        for i in 0..nx {
            let u = i as f32 / nx as f32;
            let v = (ny - j) as f32 / ny as f32;
            let r = ray(origin, lower_left_corner + u * horizontal + v * vertical);
            let col = ray_to_colour(&r, &scene);
            // let col = vec3(u, v, 0.2);
            let ir = (255.99 * col.x) as i32;
            let ig = (255.99 * col.y) as i32;
            let ib = (255.99 * col.z) as i32;
            write!(out, "{} {} {}\n", ir, ig, ib).expect("Failed to write to output.ppm");
        }
    }
}
