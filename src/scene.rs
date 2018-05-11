use rand::Rng;
use std::f32;
use std::sync::atomic::{AtomicUsize, Ordering};

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

fn random_unit_vector(rng: &mut Rng) -> Vec3 {
    let z = rng.next_f32() * 2.0 - 1.0;
    let a = rng.next_f32() * 2.0 * f32::consts::PI;
    let r = (1.0 - z * z).sqrt();
    let (sina, cosa) = a.sin_cos();
    let x = r * cosa;
    let y = r * sina;
    vec3(x, y, z)
}

#[inline]
fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * dot(v, n) * n
}

fn refract(v: Vec3, n: Vec3, ni_over_nt: f32) -> Option<Vec3> {
    let uv = normalize(v);
    let dt = dot(uv, n);
    let discriminant = 1.0 - (ni_over_nt * ni_over_nt) * (1.0 - (dt * dt));
    if discriminant > 0.0 {
        Some(ni_over_nt * (uv - n * dt) - n * discriminant.sqrt())
    } else {
        None
    }
}

#[inline]
fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

#[derive(Clone, Copy)]
pub enum Material {
    Lambertian { albedo: Vec3 },
    Metal { albedo: Vec3, fuzz: f32 },
    Dielectric { ref_idx: f32 },
}

impl Material {
    fn scatter_lambertian(
        albedo: Vec3,
        _: &Ray,
        ray_hit: &RayHit,
        rng: &mut Rng,
    ) -> Option<(Vec3, Ray)> {
        let target = ray_hit.point + ray_hit.normal + random_unit_vector(rng);
        Some((albedo, ray(ray_hit.point, target - ray_hit.point)))
    }
    fn scatter_metal(
        albedo: Vec3,
        fuzz: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut Rng,
    ) -> Option<(Vec3, Ray)> {
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
    fn scatter_dielectric(
        ref_idx: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut Rng,
    ) -> Option<(Vec3, Ray)> {
        let attenuation = vec3(1.0, 1.0, 1.0);
        let rdotn = dot(ray_in.direction, ray_hit.normal);
        let (outward_normal, ni_over_nt, cosine) = if rdotn > 0.0 {
            let cosine = rdotn / ray_in.direction.length();
            let cosine = (1.0 - ref_idx * ref_idx * (1.0 - cosine * cosine)).sqrt();
            (-ray_hit.normal, ref_idx, cosine)
        } else {
            let cosine = -rdotn / ray_in.direction.length();
            (ray_hit.normal, 1.0 / ref_idx, cosine)
        };
        if let Some(refracted) = refract(ray_in.direction, outward_normal, ni_over_nt) {
            let reflect_prob = schlick(cosine, ref_idx);
            if rng.next_f32() > reflect_prob {
                return Some((attenuation, ray(ray_hit.point, refracted)));
            }
        }
        Some((
            attenuation,
            ray(ray_hit.point, reflect(ray_in.direction, ray_hit.normal)),
        ))
    }
    fn scatter(&self, ray: &Ray, ray_hit: &RayHit, rng: &mut Rng) -> Option<(Vec3, Ray)> {
        match *self {
            Material::Lambertian { albedo } => {
                Material::scatter_lambertian(albedo, ray, ray_hit, rng)
            }
            Material::Metal { albedo, fuzz } => {
                Material::scatter_metal(albedo, fuzz, ray, ray_hit, rng)
            }
            Material::Dielectric { ref_idx } => {
                Material::scatter_dielectric(ref_idx, ray, ray_hit, rng)
            }
        }
    }
}

#[derive(Clone, Copy)]
struct RayHit {
    t: f32,
    point: Vec3,
    normal: Vec3,
}

#[derive(Clone, Copy)]
pub struct Sphere {
    pub centre: Vec3,
    pub radius: f32,
}

pub fn sphere(centre: Vec3, radius: f32, material: Material) -> (Sphere, Material) {
    (Sphere { centre, radius }, material)
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

pub struct Scene {
    spheres: Vec<Sphere>,
    materials: Vec<Material>,
    max_depth: u32,
    ray_count: AtomicUsize,
}

impl Scene {
    #[allow(dead_code)]
    pub fn random_scene(max_depth: u32, rng: &mut Rng) -> Scene {
        let n = 500;
        let mut spheres = Vec::with_capacity(n + 1);
        spheres.push(sphere(
            vec3(0.0, -1000.0, 0.0),
            1000.0,
            Material::Lambertian {
                albedo: vec3(0.5, 0.5, 0.5),
            },
        ));
        for a in -11..11 {
            for b in -11..11 {
                let choose_material = rng.next_f32();
                let centre = vec3(
                    a as f32 + 0.9 * rng.next_f32(),
                    0.2,
                    b as f32 + 0.9 * rng.next_f32(),
                );
                if choose_material < 0.8 {
                    spheres.push(sphere(
                        centre,
                        0.2,
                        Material::Lambertian {
                            albedo: vec3(
                                rng.next_f32() * rng.next_f32(),
                                rng.next_f32() * rng.next_f32(),
                                rng.next_f32() * rng.next_f32(),
                            ),
                        },
                    ));
                } else if choose_material < 0.95 {
                    spheres.push(sphere(
                        centre,
                        0.2,
                        Material::Metal {
                            albedo: vec3(
                                0.5 * (1.0 + rng.next_f32()),
                                0.5 * (1.0 + rng.next_f32()),
                                0.5 * (1.0 + rng.next_f32()),
                            ),
                            fuzz: 0.5 * rng.next_f32(),
                        },
                    ));
                } else {
                    spheres.push(sphere(centre, 0.2, Material::Dielectric { ref_idx: 1.5 }));
                }
            }
        }
        spheres.push(sphere(
            vec3(0.0, 1.0, 0.0),
            1.0,
            Material::Dielectric { ref_idx: 1.5 },
        ));
        spheres.push(sphere(
            vec3(-4.0, 1.0, 0.0),
            1.0,
            Material::Lambertian {
                albedo: vec3(0.4, 0.2, 0.1),
            },
        ));
        spheres.push(sphere(
            vec3(4.0, 1.0, 0.0),
            1.0,
            Material::Metal {
                albedo: vec3(0.7, 0.6, 0.5),
                fuzz: 0.0,
            },
        ));
        Scene::new(&spheres, max_depth)
    }

    pub fn new(sphere_materials: &[(Sphere, Material)], max_depth: u32) -> Scene {
        let (spheres, materials) = sphere_materials.iter().cloned().unzip();
        Scene {
            spheres,
            materials,
            max_depth,
            ray_count: AtomicUsize::new(0),
        }
    }

    fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material)> {
        let mut result = None;
        let mut closest_so_far = t_max;
        for (sphere, material) in self.spheres.iter().zip(self.materials.iter()) {
            if let Some(ray_hit) = sphere.hit(ray, t_min, closest_so_far) {
                closest_so_far = ray_hit.t;
                result = Some((ray_hit, material));
            }
        }
        result
    }

    pub fn ray_trace(&self, ray_in: &Ray, depth: u32, rng: &mut Rng) -> Vec3 {
        const MAX_T: f32 = f32::MAX;
        const MIN_T: f32 = 0.001;
        self.ray_count.fetch_add(1, Ordering::SeqCst);
        if let Some((ray_hit, material)) = self.ray_hit(ray_in, MIN_T, MAX_T) {
            if depth < self.max_depth {
                if let Some((attenuation, scattered)) = material.scatter(ray_in, &ray_hit, rng) {
                    return attenuation * self.ray_trace(&scattered, depth + 1, rng);
                }
            }
            return Vec3::zero();
        } else {
            let unit_direction = normalize(ray_in.direction);
            let t = 0.5 * (unit_direction.y + 1.0);
            (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0)
        }
    }

    pub fn ray_count(&self) -> usize {
        self.ray_count.load(Ordering::Relaxed)
    }
}
