use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub rcp_direction: Vec3,
}

impl Ray {
    #[inline]
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        let rcp_direction = Vec3::splat(1.0) / direction;
        Ray { origin, direction, rcp_direction }
    }

    #[inline]
    pub fn point_at_parameter(&self, t: f32) -> Vec3 {
        self.origin + (t * self.direction)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    // TODO: it would be better to calculate this lazily as not everything needs it
    pub u: f32,
    pub v: f32,
}
