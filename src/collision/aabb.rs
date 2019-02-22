#![allow(dead_code)]
use crate::collision::Ray;
use glam::{vec3, Vec3};
use std::f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        debug_assert!(!AABB::is_invalid_helper(&min, &max));
        AABB { min, max }
    }

    #[inline]
    pub fn invalid() -> AABB {
        AABB {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(-f32::MAX),
        }
    }

    #[inline]
    pub fn zero() -> AABB {
        AABB {
            min: Vec3::zero(),
            max: Vec3::zero(),
        }
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        // TODO: SIMD
        self.min.get_x() > self.max.get_x()
            || self.min.get_y() > self.max.get_y()
            || self.min.get_z() > self.max.get_z()
    }

    #[inline]
    fn is_invalid_helper(min: &Vec3, max: &Vec3) -> bool {
        // TODO: SIMD
        min.get_x() > max.get_x() || min.get_y() > max.get_y() || min.get_z() > max.get_z()
    }

    #[inline]
    pub fn ray_hit(&self, r: &Ray, tmin: f32, tmax: f32) -> bool {
        // note if not using SSE this might be faster to calc per component to early out
        let min_delta = (self.min - r.origin) / r.direction;
        let max_delta = (self.max - r.origin) / r.direction;
        let t0 = min_delta.min(max_delta);
        let t1 = min_delta.max(max_delta);
        let tmin = t0.max(Vec3::splat(tmin));
        let tmax = t1.min(Vec3::splat(tmax));
        tmax > tmin
    }

    #[inline]
    pub fn slabs(&self, p0: Vec3, p1: Vec3, ray_origin: Vec3, inv_ray_dir: Vec3) -> bool {
        let t0 = (p0 - ray_origin) * inv_ray_dir;
        let t1 = (p1 - ray_origin) * inv_ray_dir;
        let tmin = t0.min(t1);
        let tmax = t0.max(t1);
        tmin.hmax() <= tmax.hmin()
    }

    #[inline]
    pub fn add(&self, rhs: &AABB) -> AABB {
        AABB {
            min: self.min.min(rhs.min),
            max: self.max.max(rhs.max),
        }
    }

    #[inline]
    pub fn add_assign(&mut self, rhs: &AABB) {
        self.min = self.min.min(rhs.min);
        self.max = self.max.max(rhs.max);
    }

    pub fn surrounding_box(lhs: &AABB, rhs: &AABB) -> AABB {
        AABB {
            min: vec3(
                     lhs.min.get_x().min(rhs.min.get_x()),
                     lhs.min.get_y().min(rhs.min.get_y()),
                     lhs.min.get_z().min(rhs.min.get_z())),
            max: vec3(
                     lhs.max.get_x().max(rhs.max.get_x()),
                     lhs.max.get_y().max(rhs.max.get_y()),
                     lhs.max.get_z().max(rhs.max.get_z())),
        }
    }
}
