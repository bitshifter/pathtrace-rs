#![allow(dead_code)]
use crate::collision::Ray;
use glam::Vec3;
use std::f32;

#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        debug_assert!(!AABB::is_invalid_helper(min, max));
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
        self.min.cmpgt(self.max).any()
    }

    #[inline]
    fn is_invalid_helper(min: Vec3, max: Vec3) -> bool {
        min.cmpgt(max).any()
    }

    #[inline]
    pub fn ray_hit(&self, r: &Ray, tmin: f32, tmax: f32) -> bool {
        let min_delta = (self.min - r.origin) * r.rcp_direction;
        let max_delta = (self.max - r.origin) * r.rcp_direction;
        let t0 = min_delta.min(max_delta);
        let t1 = min_delta.max(max_delta);
        let tmin = t0.max(Vec3::splat(tmin));
        let tmax = t1.min(Vec3::splat(tmax));
        tmax.cmpgt(tmin).all()
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
}