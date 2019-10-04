#![allow(dead_code)]
use crate::{
    collision::{
        BVHNode, ConstantMedium, Cuboid, HitableList, Instance, MovingSphere, Ray, RayHit, Rect,
        Sphere, AABB,
    },
    material::Material,
};
use rand_xoshiro::Xoshiro256Plus;

#[derive(Copy, Clone, Debug)]
pub enum Hitable<'a> {
    BVHNode(&'a BVHNode<'a>),
    Instance(&'a Instance<'a>),
    Rect(&'a Rect, &'a Material<'a>),
    Cuboid(&'a Cuboid, &'a Material<'a>),
    MovingSphere(&'a MovingSphere, &'a Material<'a>),
    Sphere(&'a Sphere, &'a Material<'a>),
    ConstantMedium(&'a ConstantMedium<'a>),
    List(&'a HitableList<'a>),
}

impl<'a> Hitable<'a> {
    #[inline]
    pub fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        match self {
            Hitable::BVHNode(node) => Some(node.bounding_box()),
            Hitable::Instance(instance) => instance.bounding_box(t0, t1),
            Hitable::Rect(rect, _) => Some(rect.bounding_box()),
            Hitable::Cuboid(cuboid, _) => Some(cuboid.bounding_box()),
            Hitable::Sphere(sphere, _) => Some(sphere.bounding_box()),
            Hitable::MovingSphere(sphere, _) => Some(sphere.bounding_box(t0, t1)),
            Hitable::ConstantMedium(constant_medium) => constant_medium.bounding_box(t0, t1),
            Hitable::List(list) => list.bounding_box(t0, t1),
        }
    }

    #[inline]
    pub fn ray_hit(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        rng: &mut Xoshiro256Plus,
    ) -> Option<(RayHit, &Material)> {
        let (ray_hit, material) = match self {
            Hitable::BVHNode(node) => return node.ray_hit(ray, t_min, t_max, rng),
            Hitable::Instance(instance) => return instance.ray_hit(ray, t_min, t_max, rng),
            Hitable::Rect(rect, material) => (rect.ray_hit(ray, t_min, t_max), material),
            Hitable::Cuboid(cuboid, material) => (cuboid.ray_hit(ray, t_min, t_max), material),
            Hitable::Sphere(sphere, material) => (sphere.ray_hit(ray, t_min, t_max), material),
            Hitable::MovingSphere(sphere, material) => {
                (sphere.ray_hit(ray, t_min, t_max), material)
            }
            Hitable::ConstantMedium(constant_medium) => {
                return constant_medium.ray_hit(ray, t_min, t_max, rng)
            }
            Hitable::List(list) => return list.ray_hit(ray, t_min, t_max, rng),
        };
        if let Some(ray_hit) = ray_hit {
            Some((ray_hit, material))
        } else {
            None
        }
    }
}
