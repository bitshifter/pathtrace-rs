use crate::collision::{Ray, RayHit, AABB};
use glam::Vec3;
use std::f32;

// #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[derive(Clone, Copy, Debug)]
pub struct MovingSphere {
    centre_start: Vec3,
    centre_delta: Vec3,
    radius: f32,
    time_start: f32,
    inv_time_delta: f32,
}

impl MovingSphere {
    #[inline]
    pub fn new(centre0: Vec3, centre1: Vec3, time0: f32, time1: f32, radius: f32) -> MovingSphere {
        MovingSphere {
            centre_start: centre0,
            centre_delta: centre1 - centre0,
            radius,
            time_start: time0,
            inv_time_delta: 1.0 / (time1 - time0),
        }
    }

    #[inline]
    pub fn centre(&self, time: f32) -> Vec3 {
        self.centre_start + ((time - self.time_start) * self.inv_time_delta) * self.centre_delta
    }

    #[inline]
    pub fn radius(&self) -> f32 {
        self.radius
    }

    #[inline]
    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let centre = self.centre(ray.time);
        let oc = ray.origin - centre;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - a * c;
        if discriminant > 0.0 {
            let discriminant_sqrt = discriminant.sqrt();
            let t = (-b - discriminant_sqrt) / a;
            if t < t_max && t > t_min {
                let point = ray.point_at_parameter(t);
                let normal = (point - centre) / self.radius;
                return Some(RayHit {
                    point,
                    normal,
                    t,
                    u: 0.0,
                    v: 0.0,
                });
            }
            let t = (-b + discriminant_sqrt) / a;
            if t < t_max && t > t_min {
                let point = ray.point_at_parameter(t);
                let normal = (point - centre) / self.radius;
                return Some(RayHit {
                    point,
                    normal,
                    t,
                    u: 0.0,
                    v: 0.0,
                });
            }
        }
        None
    }

    #[inline]
    pub fn bounding_box(&self, t0: f32, t1: f32) -> AABB {
        let centre0 = self.centre(t0);
        let centre1 = self.centre(t1);
        let radius = Vec3::splat(self.radius);
        let aabb0 = AABB {
            min: centre0 - radius,
            max: centre0 + radius,
        };
        let aabb1 = AABB {
            min: centre1 - radius,
            max: centre1 + radius,
        };
        aabb0.add(&aabb1)
    }
}
