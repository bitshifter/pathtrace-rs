#![allow(dead_code)]
use crate::{
    collision::{BVHNode, HitableList, Ray, RayHit, Sphere, XYRect, AABB},
    material::Material,
};

#[derive(Copy, Clone, Debug)]
pub enum Hitable<'a> {
    BVHNode(&'a BVHNode<'a>),
    Sphere(&'a Sphere, &'a Material<'a>),
    XYRect(&'a XYRect, &'a Material<'a>),
    List(&'a HitableList<'a>),
}

impl<'a> Hitable<'a> {
    pub fn bounding_box(&self, t0: f32, t1: f32) -> Option<AABB> {
        match self {
            Hitable::BVHNode(node) => Some(node.bounding_box()),
            Hitable::Sphere(sphere, _) => Some(sphere.bounding_box()),
            Hitable::XYRect(rect, _) => Some(rect.bounding_box()),
            Hitable::List(list) => list.bounding_box(t0, t1),
        }
    }

    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material)> {
        let (ray_hit, material) = match self {
            Hitable::Sphere(sphere, material) => (sphere.ray_hit(ray, t_min, t_max), material),
            Hitable::XYRect(rect, material) => (rect.ray_hit(ray, t_min, t_max), material),
            Hitable::List(list) => return list.ray_hit(ray, t_min, t_max),
            Hitable::BVHNode(node) => return node.ray_hit(ray, t_min, t_max),
        };
        if let Some(ray_hit) = ray_hit {
            Some((ray_hit, material))
        } else {
            None
        }
    }
}
