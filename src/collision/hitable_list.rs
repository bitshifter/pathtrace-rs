#![allow(dead_code)]
use crate::{
    collision::{AABB, Hitable, Ray, RayHit},
    material::Material,
};
use rand_xoshiro::Xoshiro256Plus;

#[derive(Debug)]
pub struct HitableList<'a> {
    hitables: Vec<Hitable<'a>>,
}

impl<'a> HitableList<'a> {
    pub fn new(hitables: Vec<Hitable>) -> HitableList {
        HitableList { hitables }
    }

    pub fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if self.hitables.is_empty() {
            return None;
        }

        let mut result = self.hitables[0].bounding_box(t0, t1)?;

        for hitable in &self.hitables[1..] {
            let aabb = hitable.bounding_box(t0, t1)?;
            result.add_assign(&aabb);
        }

        Some(result)
    }

    pub fn ray_hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(RayHit, &Material<'_>)> {
        let mut result = None;
        let mut closest_so_far = t_max;
        for hitable in &self.hitables {
            if let Some((ray_hit, material)) = hitable.ray_hit(ray, t_min, closest_so_far, rng) {
                result = Some((ray_hit, material));
                closest_so_far = ray_hit.t;
            }
        }
        result
    }
}
