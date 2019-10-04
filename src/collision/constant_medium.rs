use crate::{
    collision::{Hitable, Ray, RayHit, AABB},
    material::{isotropic, Material},
    texture::Texture,
};
use glam::Vec3;
use rand::Rng;
use rand_xoshiro::Xoshiro256Plus;
use std::f32;

#[derive(Copy, Clone, Debug)]
pub struct ConstantMedium<'a> {
    hitable: Hitable<'a>,
    phase_function: Material<'a>,
    density: f32,
}

impl<'a> ConstantMedium<'a> {
    pub fn new(hitable: Hitable<'a>, density: f32, albedo: &'a Texture<'a>) -> Self {
        let phase_function = isotropic(albedo);
        Self {
            hitable,
            phase_function,
            density,
        }
    }

    pub fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        self.hitable.bounding_box(t0, t1)
    }

    pub fn ray_hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(RayHit, &Material)> {
        if let Some((ray_hit1, _)) = self.hitable.ray_hit(ray, -f32::MAX, f32::MAX, rng) {
            if let Some((ray_hit2, _)) =
                self.hitable
                    .ray_hit(ray, ray_hit1.t + 0.0001, f32::MAX, rng)
            {
                let mut t1 = ray_hit1.t;
                let mut t2 = ray_hit2.t;
                if t1 < t_min {
                    t1 = t_min;
                }
                if t2 > t_max {
                    t2 = t_max;
                }
                if t1 >= t2 {
                    return None;
                }
                if t1 < 0.0 {
                    t1 = 0.0;
                }
                let ray_length = ray.direction.length();
                let distance_inside_boundary = (t2 - t1) * ray_length;
                let hit_distance = -(1.0 / self.density) * rng.gen::<f32>().ln();
                if hit_distance < distance_inside_boundary {
                    let t = t1 + hit_distance / ray_length;
                    return Some((
                        RayHit {
                            point: ray.point_at_parameter(t),
                            normal: Vec3::unit_x(), // arbitrary
                            t,
                            u: 0.0,
                            v: 0.0,
                        },
                        &self.phase_function,
                    ));
                }
            }
        }
        None
    }
}
