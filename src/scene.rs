use rand::Rng;
use std::f32;
use vmath::{dot, normalize, ray, Length, Ray, Vec3, vec3};

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

#[derive(Clone, Copy)]
pub enum Material {
    Lambertian { albedo: Vec3 },
    Metal { albedo: Vec3, fuzz: f32 },
}

impl Material {
    fn scatter_lambertian(
        albedo: Vec3,
        _: &Ray,
        ray_hit: &RayHit,
        rng: &mut Rng,
    ) -> Option<(Vec3, Ray)> {
        let target = ray_hit.point + ray_hit.normal + random_in_unit_sphere(rng);
        Some((albedo, ray(ray_hit.point, target - ray_hit.point)))
    }
    fn scatter_metal(
        albedo: Vec3,
        fuzz: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut Rng,
    ) -> Option<(Vec3, Ray)> {
        let reflect = |v, n| v - 2.0 * dot(v, n) * n;
        let reflected = reflect(normalize(ray_in.direction), ray_hit.normal);
        if dot(reflected, ray_hit.normal) > 0.0 {
            Some((
                albedo,
                ray(ray_hit.point, reflected + fuzz * random_in_unit_sphere(rng)),
            ))
        } else {
            None
        }
    }
    fn scatter(&self, ray: &Ray, ray_hit: &RayHit, rng: &mut Rng) -> Option<(Vec3, Ray)> {
        match *self {
            Material::Lambertian { albedo } => {
                Material::scatter_lambertian(albedo, ray, ray_hit, rng)
            }
            Material::Metal { albedo, fuzz } => {
                Material::scatter_metal(albedo, fuzz, ray, ray_hit, rng)
            }
        }
    }
}

struct RayHit {
    t: f32,
    point: Vec3,
    normal: Vec3,
    material: Material,
}

pub struct Sphere {
    pub centre: Vec3,
    pub radius: f32,
    pub material: Material,
}

pub fn sphere(centre: Vec3, radius: f32, material: Material) -> Sphere {
    Sphere {
        centre,
        radius,
        material,
    }
}

impl Sphere {
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
                return Some(RayHit {
                    t,
                    point,
                    normal,
                    material: self.material,
                });
            }
            let t = (-b + discriminant_sqrt) / a;
            if t < t_max && t > t_min {
                let point = ray.point_at_parameter(t);
                let normal = (point - self.centre) / self.radius;
                return Some(RayHit {
                    t,
                    point,
                    normal,
                    material: self.material,
                });
            }
        }
        None
    }
}

pub struct Scene {
    spheres: Vec<Sphere>,
}

impl Scene {
    pub fn new(spheres: Vec<Sphere>) -> Scene {
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
    pub fn ray_to_colour(&self, ray_in: &Ray, depth: u32, rng: &mut Rng) -> Vec3 {
        if let Some(ray_hit) = self.hit(ray_in, 0.001, f32::MAX) {
            if depth >= 50 {
                Vec3::zero()
            } else if let Some((attenuation, scattered)) =
                ray_hit.material.scatter(ray_in, &ray_hit, rng)
            {
                attenuation * self.ray_to_colour(&scattered, depth + 1, rng)
            } else {
                Vec3::zero()
            }
        } else {
            let unit_direction = normalize(ray_in.direction);
            let t = 0.5 * (unit_direction.y + 1.0);
            (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0)
        }
    }
}
