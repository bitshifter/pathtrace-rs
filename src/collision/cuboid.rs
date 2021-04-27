use crate::collision::{Ray, RayHit, Rect, AABB};
use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Cuboid {
    faces: [Rect; 6],
    aabb: AABB,
}

impl Cuboid {
    pub fn new(p0: Vec3, p1: Vec3) -> Cuboid {
        Cuboid {
            faces: [
                Rect::new_xy(p0.x, p1.x, p0.y, p1.y, p1.z, false),
                Rect::new_xy(p0.x, p1.x, p0.y, p1.y, p0.z, true),
                Rect::new_xz(p0.x, p1.x, p0.z, p1.z, p1.y, false),
                Rect::new_xz(p0.x, p1.x, p0.z, p1.z, p0.y, true),
                Rect::new_yz(p0.y, p1.y, p0.z, p1.z, p1.x, false),
                Rect::new_yz(p0.y, p1.y, p0.z, p1.z, p0.x, true),
            ],
            aabb: AABB::new(p0, p1),
        }
    }

    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        let mut result = None;
        if self.aabb.ray_hit(ray, t_min, t_max) {
            let mut closest_so_far = t_max;
            for face in &self.faces {
                if let Some(ray_hit) = face.ray_hit(ray, t_min, closest_so_far) {
                    result = Some(ray_hit);
                    closest_so_far = ray_hit.t;
                }
            }
        }
        result
    }

    pub fn bounding_box(&self) -> AABB {
        self.aabb
    }
}
