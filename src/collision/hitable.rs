#![allow(dead_code)]
use crate::{collision::{AABB, Ray, RayHit, Sphere, XYRect}, material::Material};
use rand::{Rng, XorShiftRng};
use typed_arena::Arena;

#[derive(Copy, Clone, Debug)]
pub struct BVHNode<'a> {
    aabb: AABB,
    lhs: Hitable<'a>,
    rhs: Hitable<'a>,
}

#[inline]
fn alloc_from_nodes<'a>(arena: &'a Arena<BVHNode<'a>>, lhs: &'a BVHNode<'a>, rhs: &'a BVHNode<'a>, aabb: AABB) -> &'a BVHNode<'a>  {
    arena.alloc(
        BVHNode {
            aabb,
            lhs: Hitable::BVHNode(lhs),
            rhs: Hitable::BVHNode(rhs),
        })
}

#[inline]
fn alloc_from_hittables<'a>(arena: &'a Arena<BVHNode<'a>>, lhs: Hitable<'a>, rhs: Hitable<'a>, aabb: AABB) -> &'a BVHNode<'a>  {
    arena.alloc(
        BVHNode {
            aabb,
            lhs: lhs,
            rhs: rhs,
        })
}
impl<'a> BVHNode<'a> {
    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material)> {
        if self.aabb.ray_hit(ray, t_min, t_max) {
            let hit_lhs = self.lhs.ray_hit(ray, t_min, t_max);
            let hit_rhs = self.rhs.ray_hit(ray, t_min, t_max);
            match (hit_lhs, hit_rhs) {
                (Some(hit_lhs), Some(hit_rhs)) => {
                    if hit_lhs.0.t < hit_rhs.0.t {
                        Some(hit_lhs)
                    } else {
                        Some(hit_rhs)
                    }
                }
                (Some(hit_lhs), None) => Some(hit_lhs),
                (None, Some(hit_rhs)) => Some(hit_rhs),
                (None, None) => {
                    unreachable!()
                }
            }
        } else {
            None
        }
    }

    pub fn new(rng: &mut XorShiftRng, hitables: &mut [Hitable<'a>], arena: &'a Arena<BVHNode<'a>>) -> Option<&'a BVHNode<'a>> {
        let root = BVHNode::new_split(rng, hitables, arena, 0.0, 0.0);
        // dbg!(&root);
        root
    }

    pub fn new_split(
        rng: &mut XorShiftRng,
        hitables: &mut [Hitable<'a>],
        arena: &'a Arena<BVHNode<'a>>,
        t0: f32,
        t1: f32,
    ) -> Option<&'a BVHNode<'a>> {
        let axis = rng.next_u32() % 3;
        hitables.sort_unstable_by(|lhs, rhs| {
            let lhs_min = lhs.bounding_box(t0, t1).unwrap().min;
            let rhs_min = rhs.bounding_box(t0, t1).unwrap().min;
            let ord = match axis {
                0 => lhs_min.get_x().partial_cmp(&rhs_min.get_x()),
                1 => lhs_min.get_y().partial_cmp(&rhs_min.get_y()),
                2 => lhs_min.get_z().partial_cmp(&rhs_min.get_z()),
                _ => unreachable!(),
            };
            ord.unwrap()
        });
        match hitables.len() {
            0 => None,
            1 => {
                let lhs = hitables[0];
                let rhs = lhs;
                let aabb = lhs.bounding_box(t0, t1).unwrap();
                Some(alloc_from_hittables(arena, lhs, rhs, aabb))
            }
            2 => {
                let lhs = hitables[0];
                let rhs = hitables[1];
                let lhs_aabb = lhs.bounding_box(t0, t1).unwrap();
                let rhs_aabb = rhs.bounding_box(t0, t1).unwrap();
                let aabb = lhs_aabb.add(&rhs_aabb);
                Some(alloc_from_hittables(arena, lhs, rhs, aabb))
            }
            _ => {
                let pivot = hitables.len() / 2;
                let lhs =
                    BVHNode::new_split(rng, &mut hitables[0..pivot], arena, t0, t1).unwrap();
                let rhs =
                    BVHNode::new_split(rng, &mut hitables[pivot + 1..], arena, t0, t1).unwrap();
                let lhs_aabb = lhs.aabb;
                let rhs_aabb = rhs.aabb;
                let aabb = lhs_aabb.add(&rhs_aabb);
                Some(alloc_from_nodes(arena, lhs, rhs, aabb))
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Hitable<'a> {
    BVHNode(&'a BVHNode<'a>),
    Sphere(&'a Sphere, &'a Material<'a>),
    XYRect(&'a XYRect, &'a Material<'a>),
}

impl<'a> Hitable<'a> {
    pub fn bounding_box(&self, _t0: f32, _t1: f32) -> Option<AABB> {
        match self {
            Hitable::BVHNode(node) => Some(node.aabb),
            Hitable::Sphere(sphere, _) => Some(sphere.bounding_box()),
            Hitable::XYRect(rect, _) => Some(rect.bounding_box()),
        }
    }

    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material)> {
        let (ray_hit, material) = match self {
            Hitable::Sphere(sphere, material) => (sphere.ray_hit(ray, t_min, t_max), material),
            Hitable::XYRect(rect, material) => (rect.ray_hit(ray, t_min, t_max), material),
            Hitable::BVHNode(_) => unreachable!(),
        };
        if let Some(ray_hit) = ray_hit {
            Some((ray_hit, material))
        } else {
            None
        }
    }
}


