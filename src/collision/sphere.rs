use crate::collision::{Ray, RayHit, AABB};
use glam::Vec3;
use std::f32;

// #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub centre: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(centre: Vec3, radius: f32) -> Sphere {
        Sphere { centre, radius }
    }

    #[inline]
    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let oc = ray.origin - self.centre;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - a * c;
        if discriminant > 0.0 {
            let discriminant_sqrt = discriminant.sqrt();
            let t = (-b - discriminant_sqrt) / a;
            if t < t_max && t > t_min {
                let point = ray.point_at_parameter(t);
                let normal = (point - self.centre) / self.radius;
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
                let normal = (point - self.centre) / self.radius;
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
    pub fn bounding_box(&self) -> AABB {
        let radius = Vec3::splat(self.radius);
        AABB {
            min: self.centre - radius,
            max: self.centre + radius,
        }
    }
}
