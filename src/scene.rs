use rand::Rng;
use std::f32;
use vmath::{dot, normalize, Length, Ray, Sphere, Vec3, vec3};

pub struct RayHit {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

trait Hitable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit>;
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

fn random_in_unit_sphere(rng: &mut Rng) -> Vec3 {
    loop {
        let p = vec3(
            2.0 * rng.next_f32() - 1.0,
            2.0 * rng.next_f32() - 1.0,
            2.0 * rng.next_f32() - 1.0,
        );
        if p.length_squared() < 1.0 {
            return p;
        }
    }
}

pub struct Scene {
    spheres: Vec<Sphere>,
}

impl Scene {
    pub fn new(spheres: Vec<Sphere>) -> Scene {
        Scene { spheres }
    }
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
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
    pub fn ray_to_colour(&self, ray: &Ray, rng: &mut Rng) -> Vec3 {
        if let Some(ray_hit) = self.hit(ray, 0.001, f32::MAX) {
            let target = ray_hit.point + ray_hit.normal + random_in_unit_sphere(rng);
            0.5 * self.ray_to_colour(&Ray::new(ray_hit.point, target - ray_hit.point), rng)
        } else {
            let unit_direction = normalize(ray.direction);
            let t = 0.5 * (unit_direction.y + 1.0);
            (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0)
        }
    }
}
