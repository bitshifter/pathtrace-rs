use collision::AABB;

enum Hitable {
    Sphere { index: u32 }
}

// trait Hitable {
//     fn hit(&self, ray: &Ray, t_min: f32, t_max: f32)
// }

struct Node {
    left: Hitable,
    right: Hitable,
    box: AABB,
}

impl Node {
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        if self.box.hit(ray, t_min, t_max) {
            let hit_left = self.left.hit(ray, t_min, t_max);
            let hit_right = self.right.hit(ray, t_min, t_max);
        }
    }
}
