#![allow(dead_code)]
use crate::collision::Ray;
use glam::{Mat4, Vec3, Vec3A};
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
        let min = Vec3A::from(self.min);
        let max = Vec3A::from(self.max);
        let origin = Vec3A::from(r.origin);
        let rcp_direction = Vec3A::from(r.rcp_direction);
        let min_delta = (min - origin) * rcp_direction;
        let max_delta = (max - origin) * rcp_direction;
        let t0 = min_delta.min(max_delta);
        let t1 = min_delta.max(max_delta);
        let tmin = t0.max(Vec3A::splat(tmin));
        let tmax = t1.min(Vec3A::splat(tmax));
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

    #[inline]
    pub fn transform(&self, m: &Mat4) -> Self {
        let min = m.w_axis().truncate();
        let max = min;

        let mut min_out = min;
        let mut max_out = max;

        let x_axis = m.x_axis().truncate();
        let x_mask = x_axis.cmpgt(Vec3A::zero());
        let y_axis = m.y_axis().truncate();
        let y_mask = y_axis.cmpgt(Vec3A::zero());
        let z_axis = m.z_axis().truncate();
        let z_mask = z_axis.cmpgt(Vec3A::zero());

        min_out += x_axis * x_mask.select(min, max);
        max_out += x_axis * x_mask.select(max, min);
        min_out += y_axis * y_mask.select(min, max);
        max_out += y_axis * y_mask.select(max, min);
        min_out += z_axis * z_mask.select(min, max);
        max_out += z_axis * z_mask.select(max, min);

        AABB { min: Vec3::from(min_out), max: Vec3::from(max_out) }
    }
}
