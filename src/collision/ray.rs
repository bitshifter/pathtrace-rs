use glam::{Mat4, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub rcp_direction: Vec3,
    pub time: f32,
}

impl Ray {
    #[inline]
    pub fn new(origin: Vec3, direction: Vec3, time: f32) -> Self {
        let rcp_direction = direction.recip();
        Ray {
            origin,
            direction,
            rcp_direction,
            time,
        }
    }

    #[inline]
    pub fn point_at_parameter(&self, t: f32) -> Vec3 {
        self.origin + (t * self.direction)
    }

    #[inline]
    pub fn transform(&self, m: &Mat4) -> Self {
        let origin = m.transform_point3(self.origin);
        let direction = m.transform_vector3(self.direction);
        let rcp_direction = direction.recip();
        Ray {
            origin,
            direction: direction,
            rcp_direction: rcp_direction,
            time: self.time,
        }
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

impl RayHit {
    #[inline]
    pub fn transform(&self, m: &Mat4) -> Self {
        let point = m.transform_point3(self.point);
        let normal = m.transform_vector3(self.normal);
        RayHit {
            point,
            normal,
            t: self.t,
            u: self.u,
            v: self.v,
        }
    }
}
