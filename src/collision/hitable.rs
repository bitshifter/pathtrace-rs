#![allow(dead_code)]
use crate::{
    collision::{BVHNode, Cuboid, HitableList, MovingSphere, Ray, RayHit, Rect, Sphere, AABB},
    material::Material,
};

#[derive(Copy, Clone, Debug)]
pub enum Hitable<'a> {
    BVHNode(&'a BVHNode<'a>),
    MovingSphere(&'a MovingSphere, &'a Material<'a>),
    Sphere(&'a Sphere, &'a Material<'a>),
    Rect(&'a Rect, &'a Material<'a>),
    Cuboid(&'a Cuboid, &'a Material<'a>),
    List(&'a HitableList<'a>),
}

impl<'a> Hitable<'a> {
    #[inline]
    pub fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        match self {
            Hitable::BVHNode(node) => Some(node.bounding_box()),
            Hitable::MovingSphere(sphere, _) => Some(sphere.bounding_box(t0, t1)),
            Hitable::Sphere(sphere, _) => Some(sphere.bounding_box()),
            Hitable::Rect(rect, _) => Some(rect.bounding_box()),
            Hitable::Cuboid(cuboid, _) => Some(cuboid.bounding_box()),
            Hitable::List(list) => list.bounding_box(t0, t1),
        }
    }

    #[inline]
    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material)> {
        let (ray_hit, material) = match self {
            Hitable::BVHNode(node) => return node.ray_hit(ray, t_min, t_max),
            Hitable::MovingSphere(sphere, material) => {
                (sphere.ray_hit(ray, t_min, t_max), material)
            }
            Hitable::Sphere(sphere, material) => (sphere.ray_hit(ray, t_min, t_max), material),
            Hitable::Rect(rect, material) => (rect.ray_hit(ray, t_min, t_max), material),
            Hitable::Cuboid(cuboid, material) => (cuboid.ray_hit(ray, t_min, t_max), material),
            Hitable::List(list) => return list.ray_hit(ray, t_min, t_max),
        };
        if let Some(ray_hit) = ray_hit {
            Some((ray_hit, material))
        } else {
            None
        }
    }
}
