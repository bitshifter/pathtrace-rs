use crate::{
    collision::{Ray, RayHit},
    math::{random_in_unit_sphere, random_unit_vector, reflect, refract, schlick},
    texture::Texture,
};
use glam::{vec3, Vec3};
use rand::Rng;
use rand_xoshiro::Xoshiro256Plus;
use std::f32;

// #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[derive(Clone, Copy, Debug)]
pub enum Material<'a> {
    Lambertian { albedo: &'a Texture<'a> },
    Metal { albedo: Vec3, fuzz: f32 },
    Dielectric { ref_idx: f32 },
    DiffuseLight { emit: &'a Texture<'a> },
}

pub fn lambertian<'a>(albedo: &'a Texture<'a>) -> Material<'a> {
    Material::Lambertian { albedo }
}

pub fn metal<'a>(albedo: Vec3, fuzz: f32) -> Material<'a> {
    Material::Metal { albedo, fuzz }
}

pub fn dielectric<'a>(ref_idx: f32) -> Material<'a> {
    Material::Dielectric { ref_idx }
}

pub fn diffuse_light<'a>(emit: &'a Texture<'a>) -> Material<'a> {
    Material::DiffuseLight { emit }
}

fn get_sphere_uv(normal: Vec3) -> (f32, f32) {
    const FRAC_1_2PI: f32 = 1.0 / (2.0 * f32::consts::PI);
    let (x, y, _) = normal.into();
    let phi = x.atan2(y);
    let theta = y.asin();
    let u = 1.0 - (phi + f32::consts::PI) * FRAC_1_2PI;
    let v = (theta + f32::consts::FRAC_PI_2) * f32::consts::FRAC_1_PI;
    return (u, v);
}

impl<'a> Material<'a> {
    fn scatter_lambertian(
        albedo: &Texture,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(Vec3, Ray)> {
        let target = ray_hit.point + ray_hit.normal + random_unit_vector(rng);
        Some((
            albedo.value(ray_hit.u, ray_hit.v, ray_hit.point),
            Ray::new(
                ray_hit.point,
                (target - ray_hit.point).normalize(),
                ray_in.time,
            ),
        ))
    }

    fn scatter_metal(
        albedo: &Vec3,
        fuzz: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(Vec3, Ray)> {
        let reflected = reflect(ray_in.direction, ray_hit.normal);
        if reflected.dot(ray_hit.normal) > 0.0 {
            Some((
                *albedo,
                Ray::new(
                    ray_hit.point,
                    (reflected + fuzz * random_in_unit_sphere(rng)).normalize(),
                    ray_in.time,
                ),
            ))
        } else {
            None
        }
    }

    fn scatter_dielectric(
        ref_idx: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(Vec3, Ray)> {
        let attenuation = vec3(1.0, 1.0, 1.0);
        let rdotn = ray_in.direction.dot(ray_hit.normal);
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
            if rng.gen::<f32>() > reflect_prob {
                return Some((
                    attenuation,
                    Ray::new(ray_hit.point, refracted.normalize(), ray_in.time),
                ));
            }
        }
        Some((
            attenuation,
            Ray::new(
                ray_hit.point,
                reflect(ray_in.direction, ray_hit.normal).normalize(),
                ray_in.time,
            ),
        ))
    }

    pub fn scatter(
        &self,
        ray: &Ray,
        ray_hit: &RayHit,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(Vec3, Ray)> {
        match self {
            Material::Lambertian { albedo } => {
                Material::scatter_lambertian(albedo, ray, ray_hit, rng)
            }
            Material::Metal { albedo, fuzz } => {
                Material::scatter_metal(albedo, *fuzz, ray, ray_hit, rng)
            }
            Material::Dielectric { ref_idx } => {
                Material::scatter_dielectric(*ref_idx, ray, ray_hit, rng)
            }
            Material::DiffuseLight { emit: _ } => None,
        }
    }

    pub fn emitted(&self, u: f32, v: f32, point: Vec3) -> Vec3 {
        if let Material::DiffuseLight { emit } = self {
            emit.value(u, v, point)
        } else {
            Vec3::zero()
        }
    }

    pub fn get_sphere_uv(&self, normal: Vec3) -> (f32, f32) {
        if let Material::Lambertian { albedo } = self {
            if let Texture::Image { image: _ } = albedo {
                return get_sphere_uv(normal);
            }
        } else if let Material::DiffuseLight { emit } = self {
            if let Texture::Image { image: _ } = emit {
                return get_sphere_uv(normal);
            }
        }
        (0.0, 0.0)
    }
}
