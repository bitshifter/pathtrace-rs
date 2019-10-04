#![allow(dead_code)]
use crate::{
    collision::{Hitable, Ray, RayHit, AABB},
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

        let mut result = if let Some(aabb) = self.hitables[0].bounding_box(t0, t1) {
            aabb
        } else {
            return None;
        };

        for hitable in &self.hitables[1..] {
            if let Some(aabb) = hitable.bounding_box(t0, t1) {
                result.add_assign(&aabb);
            } else {
                return None;
            }
        }

        Some(result)
    }

    pub fn ray_hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(RayHit, &Material)> {
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

#[cfg(all(feature = "bench", test))]
mod bench {
    use crate::{
        bench::hitables_bench,
        collision::HitableList,
        scene::{MAX_T, MIN_T},
    };
    use test::Bencher;

    #[bench]
    fn ray_hit(b: &mut Bencher) {
        hitables_bench(|ray, hitables| {
            let list = HitableList::new(hitables);
            b.iter(|| list.ray_hit(&ray, MIN_T, MAX_T));
        })
    }
}
