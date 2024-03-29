use crate::{
    collision::{Hitable, Ray, RayHit, AABB},
    material::Material,
};
use glam::Affine3A;
use rand_xoshiro::Xoshiro256Plus;

#[derive(Copy, Clone, Debug)]
pub struct Instance<'a> {
    hitable: Hitable<'a>,
    transform: Affine3A,
    inv_transform: Affine3A,
}

impl<'a> Instance<'a> {
    pub fn new(hitable: Hitable<'a>, transform: Affine3A) -> Instance<'a> {
        Instance {
            hitable,
            transform,
            inv_transform: transform.inverse(),
        }
    }

    pub fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        if let Some(aabb) = self.hitable.bounding_box(t0, t1) {
            Some(aabb.transform(&self.transform))
        } else {
            None
        }
    }

    pub fn ray_hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(RayHit, &Material)> {
        if let Some((ray_hit, material)) =
            self.hitable
                .ray_hit(&ray.transform(&self.inv_transform), t_min, t_max, rng)
        {
            Some((ray_hit.transform(&self.transform), material))
        } else {
            None
        }
    }
}
