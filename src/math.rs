use rand::Rng;
use std::f32;
use vmath::{dot, Length, vec3, Vec3};

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
    let x = r * cosa;
    let y = r * sina;
    vec3(x, y, z)
}

