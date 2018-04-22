mod vmath;

use std::fs::File;
use std::io::Write;

use vmath::{dot, normalize, ray, vec3, Ray, Vec3};

fn color(ray: &Ray) -> Vec3 {
    if hit_sphere(vec3(0.0, 0.0, -1.0), 0.5, ray) {
        return vec3(1.0, 0.0, 0.0);
    }
    let unit_direction = normalize(ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0)
}

fn hit_sphere(centre: Vec3, radius: f32, r: &Ray) -> bool {
    let oc = r.origin - centre;
    let a = dot(r.direction, r.direction);
    let b = 2.0 * dot(oc, r.direction);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a *c;
    discriminant > 0.0
}

fn main() {
    let nx = 200;
    let ny = 100;
    let lower_left_corner = vec3(-2.0, -1.0, -1.0);
    let horizontal = vec3(4.0, 0.0, 0.0);
    let vertical = vec3(0.0, 2.0, 0.0);
    let origin = vec3(0.0, 0.0, 0.0);
    let mut out = File::create("output.ppm").expect("Failed to create output.ppm");
    write!(out, "P3\n{} {} \n255\n", nx, ny).expect("Failed to write to output.ppm");
    for j in 0..(ny - 1) {
        for i in 0..nx {
            let u = i as f32 / nx as f32;
            let v = (ny - j) as f32 / ny as f32;
            let r = ray(origin, lower_left_corner + u * horizontal + v * vertical);
            let col = color(&r);
            // let col = vec3(u, v, 0.2);
            let ir = (255.99 * col.x) as i32;
            let ig = (255.99 * col.y) as i32;
            let ib = (255.99 * col.z) as i32;
            write!(out, "{} {} {}\n", ir, ig, ib).expect("Failed to write to output.ppm");
        }
    }
}
