#![allow(dead_code)]
use crate::collision::{Ray, RayHit, AABB};
use glam::vec3;

#[derive(Copy, Clone, Debug)]
pub struct XYRect {
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    k: f32,
}

impl XYRect {
    pub fn new(x0: f32, x1: f32, y0: f32, y1: f32, k: f32) -> XYRect {
        XYRect { x0, x1, y0, y1, k }
    }

    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let t = (self.k - ray.origin.get_z()) / ray.direction.get_z();
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.get_x() + t * ray.direction.get_x();
        let y = ray.origin.get_y() + t * ray.direction.get_y();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        Some(RayHit {
            point: ray.point_at_parameter(t),
            normal: vec3(0.0, 0.0, 1.0),
            t,
            u: (x - self.x0) / (self.x1 - self.x0),
            v: (y - self.y0) / (self.y1 - self.y0),
        })
    }

    pub fn bounding_box(&self) -> AABB {
        AABB {
            min: vec3(self.x0, self.y0, self.k - 0.0001),
            max: vec3(self.x1, self.y1, self.k + 0.0001),
        }
    }
}
