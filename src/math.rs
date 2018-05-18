use rand::Rng;
use std::f32;
use vmath::{dot, normalize, vec3, Length, Vec3};

pub fn random_in_unit_disk<T: Rng>(rng: &mut T) -> Vec3 {
    loop {
        let p = 2.0 * vec3(rng.next_f32(), rng.next_f32(), 0.0) - vec3(1.0, 1.0, 0.0);
        if dot(p, p) < 1.0 {
            return p;
        }
    }
}

pub fn random_in_unit_sphere<T: Rng>(rng: &mut T) -> Vec3 {
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

pub fn random_unit_vector<T: Rng>(rng: &mut T) -> Vec3 {
    let z = rng.next_f32() * 2.0 - 1.0;
    let a = rng.next_f32() * 2.0 * f32::consts::PI;
    let r = (1.0 - z * z).sqrt();
    let (sina, cosa) = a.sin_cos();
    vec3(r * cosa, r * sina, z)
}

pub fn linear_to_srgb(rgb: (f32, f32, f32)) -> (u8, u8, u8) {
    let rgb = (rgb.0.max(0.0), rgb.1.max(0.0), rgb.2.max(0.0));
    let srgb = (
        (1.055 * rgb.0.powf(0.416666667) - 0.055).max(0.0),
        (1.055 * rgb.1.powf(0.416666667) - 0.055).max(0.0),
        (1.055 * rgb.2.powf(0.416666667) - 0.055).max(0.0),
    );
    (
        (srgb.0 * 255.99) as u8,
        (srgb.1 * 255.99) as u8,
        (srgb.2 * 255.99) as u8,
    )
}

#[inline]
pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * dot(v, n) * n
}

pub fn refract(v: Vec3, n: Vec3, ni_over_nt: f32) -> Option<Vec3> {
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
pub fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}
