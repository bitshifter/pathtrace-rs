use crate::{
    collision::{ray, Ray, RayHit},
    math::{random_in_unit_sphere, random_unit_vector, reflect, refract, schlick},
    texture::Texture,
};
use glam::{vec3, Vec3};
use rand::{Rng, XorShiftRng};
use std::f32;

// #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[derive(Clone, Copy, Debug)]
pub enum MaterialKind<'a> {
    Lambertian { albedo: &'a Texture<'a> },
    Metal { albedo: Vec3, fuzz: f32 },
    Dielectric { ref_idx: f32 },
}

#[derive(Clone, Copy, Debug)]
pub struct Material<'a> {
    pub kind: MaterialKind<'a>,
    pub emissive: Vec3,
}

pub fn lambertian<'a>(albedo: &'a Texture<'a>) -> Material<'a> {
    Material {
        kind: MaterialKind::Lambertian { albedo },
        emissive: Vec3::zero(),
    }
}

pub fn lambertian_emissive<'a>(albedo: &'a Texture<'a>, emissive: Vec3) -> Material<'a> {
    Material {
        kind: MaterialKind::Lambertian { albedo },
        emissive,
    }
}

pub fn metal<'a>(albedo: Vec3, fuzz: f32) -> Material<'a> {
    Material {
        kind: MaterialKind::Metal { albedo, fuzz },
        emissive: Vec3::zero(),
    }
}

pub fn dielectric<'a>(ref_idx: f32) -> Material<'a> {
    Material {
        kind: MaterialKind::Dielectric { ref_idx },
        emissive: Vec3::zero(),
    }
}

impl<'a> MaterialKind<'a> {
    fn scatter_lambertian(
        albedo: &Texture,
        _: &Ray,
        ray_hit: &RayHit,
        rng: &mut XorShiftRng,
    ) -> Option<(Vec3, Ray, bool)> {
        let target = ray_hit.point + ray_hit.normal + random_unit_vector(rng);
        Some((
            albedo.value(ray_hit.u, ray_hit.v, ray_hit.point),
            ray(ray_hit.point, (target - ray_hit.point).normalize()),
            true,
        ))
    }
    fn scatter_metal(
        albedo: Vec3,
        fuzz: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut XorShiftRng,
    ) -> Option<(Vec3, Ray, bool)> {
        let reflected = reflect(ray_in.direction, ray_hit.normal);
        if reflected.dot(ray_hit.normal) > 0.0 {
            Some((
                albedo,
                ray(
                    ray_hit.point,
                    (reflected + fuzz * random_in_unit_sphere(rng)).normalize(),
                ),
                false,
            ))
        } else {
            None
        }
    }
    fn scatter_dielectric(
        ref_idx: f32,
        ray_in: &Ray,
        ray_hit: &RayHit,
        rng: &mut XorShiftRng,
    ) -> Option<(Vec3, Ray, bool)> {
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
            if rng.next_f32() > reflect_prob {
                return Some((
                    attenuation,
                    ray(ray_hit.point, refracted.normalize()),
                    false,
                ));
            }
        }
        Some((
            attenuation,
            ray(
                ray_hit.point,
                reflect(ray_in.direction, ray_hit.normal).normalize(),
            ),
            false,
        ))
    }
}

impl<'a> Material<'a> {
    pub fn scatter(
        &self,
        ray: &Ray,
        ray_hit: &RayHit,
        rng: &mut XorShiftRng,
    ) -> Option<(Vec3, Ray, bool)> {
        match self.kind {
            MaterialKind::Lambertian { albedo } => {
                MaterialKind::scatter_lambertian(albedo, ray, ray_hit, rng)
            }
            MaterialKind::Metal { albedo, fuzz } => {
                MaterialKind::scatter_metal(albedo, fuzz, ray, ray_hit, rng)
            }
            MaterialKind::Dielectric { ref_idx } => {
                MaterialKind::scatter_dielectric(ref_idx, ray, ray_hit, rng)
            }
        }
    }
    pub fn get_sphere_uv(&self, p: Vec3) -> (f32, f32) {
        if let MaterialKind::Lambertian { albedo } = self.kind {
            if let Texture::Image { image: _ } = albedo {
                const FRAC_1_2PI: f32 = 1.0 / (2.0 * f32::consts::PI);
                let phi = p.get_x().atan2(p.get_y());
                let theta = p.get_y().asin();
                let u = 1.0 - (phi + f32::consts::PI) * FRAC_1_2PI;
                let v = (theta + f32::consts::FRAC_PI_2) * f32::consts::FRAC_1_PI;
                return (u, v);
            }
        }
        (0.0, 0.0)
    }

}
