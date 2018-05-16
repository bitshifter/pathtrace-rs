use rand::Rng;
use std::f32;
use vmath::{dot, Length, Vec3, vec3};

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
        (1.055 * rgb.2.powf(0.416666667) - 0.055).max(0.0)
        );
    ((srgb.0 * 255.99) as u8, (srgb.1 * 255.99) as u8, (srgb.2 * 255.99) as u8)
}

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[inline]
pub fn ray(origin: Vec3, direction: Vec3) -> Ray {
    Ray { origin, direction }
}

impl Ray {
    #[inline]
    #[allow(dead_code)]
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }
    #[inline]
    pub fn point_at_parameter(&self, t: f32) -> Vec3 {
        self.origin + (t * self.direction)
    }
}
