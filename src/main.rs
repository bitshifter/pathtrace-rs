extern crate image;
extern crate rand;

mod vmath;

use image::RgbImage;
use rand::Rng;

use vmath::{dot, normalize, ray, Ray, Vec3, vec3};

struct RayHit {
    t: f32,
    point: Vec3,
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
                let point = ray.point_at_parameter(t);
                let normal = (point - self.centre) / self.radius;
                return Some(RayHit { t, point, normal });
            }
            let t = (-b + discriminant_sqrt) / a;
            if t < t_max && t > t_min {
                let point = ray.point_at_parameter(t);
                let normal = (point - self.centre) / self.radius;
                return Some(RayHit { t, point, normal });
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

struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    fn default() -> Camera {
        Camera {
            lower_left_corner: vec3(-2.0, -1.0, -1.0),
            horizontal: vec3(4.0, 0.0, 0.0),
            vertical: vec3(0.0, 2.0, 0.0),
            origin: vec3(0.0, 0.0, 0.0),
        }
    }

    fn get_ray(&self, u: f32, v: f32) -> Ray {
        ray(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical,
        )
    }
}

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
    let ns = 100;
    let scene = Scene::new(vec![
        sphere(vec3(0.0, 0.0, -1.0), 0.5),
        sphere(vec3(0.0, -100.5, -1.0), 100.0),
    ]);
    let camera = Camera::default();
    let mut rng = rand::weak_rng();
    let mut img = RgbImage::new(nx, ny);
    for j in 0..(ny - 1) {
        for i in 0..nx {
            let mut col = vec3(0.0, 0.0, 0.0);
            for _ in 0..ns {
                let u = (i as f32 + rng.next_f32()) / nx as f32;
                let v = ((ny - j) as f32 + rng.next_f32()) / ny as f32;
                let ray = camera.get_ray(u, v);
                // let point = ray.point_at_parameter(2.0);
                col += ray_to_colour(&ray, &scene);
            }
            col /= ns as f32;
            let pixel = &mut img[(i, j)];
            pixel[0] = (255.99 * col.x) as u8;
            pixel[1] = (255.99 * col.y) as u8;
            pixel[2] = (255.99 * col.z) as u8;
        }
    }
    img.save("output.png").expect("Failed to save output image");
}
