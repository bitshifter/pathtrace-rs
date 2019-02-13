use crate::simd::sinf_cosf;
use glam::{vec3, Vec3};
use rand::Rng;
use std::f32;

pub fn random_in_unit_disk<T: Rng>(rng: &mut T) -> Vec3 {
    loop {
        let p = 2.0 * vec3(rng.next_f32(), rng.next_f32(), 0.0) - vec3(1.0, 1.0, 0.0);
        if p.dot(p) < 1.0 {
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
    let (sina, cosa) = sinf_cosf(a);
    vec3(r * cosa, r * sina, z)
}

pub fn linear_to_srgb(rgb: (f32, f32, f32)) -> (u8, u8, u8) {
    let rgb = (rgb.0.max(0.0), rgb.1.max(0.0), rgb.2.max(0.0));
    let srgb = (
        (1.055 * rgb.0.powf(0.416_666_66) - 0.055).max(0.0).min(1.0),
        (1.055 * rgb.1.powf(0.416_666_66) - 0.055).max(0.0).min(1.0),
        (1.055 * rgb.2.powf(0.416_666_66) - 0.055).max(0.0).min(1.0),
    );
    (
        (srgb.0 * 255.99) as u8,
        (srgb.1 * 255.99) as u8,
        (srgb.2 * 255.99) as u8,
    )
}

#[inline]
pub fn maxf(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

#[inline]
pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

pub fn refract(v: Vec3, n: Vec3, ni_over_nt: f32) -> Option<Vec3> {
    let dt = v.dot(n);
    let discriminant = 1.0 - (ni_over_nt * ni_over_nt) * (1.0 - (dt * dt));
    if discriminant > 0.0 {
        Some(ni_over_nt * (v - n * dt) - n * discriminant.sqrt())
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

#[inline]
pub fn align_to(value: usize, align: usize) -> usize {
    (value + (align - 1)) & !(align - 1)
}
