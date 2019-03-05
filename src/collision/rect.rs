#![allow(dead_code)]
use crate::collision::{Ray, RayHit, AABB};
use glam::vec3;

#[derive(Copy, Clone, Debug)]
pub enum Rect {
    XY {
        x0: f32,
        x1: f32,
        y0: f32,
        y1: f32,
        k: f32,
        flip_normals: bool,
    },
    XZ {
        x0: f32,
        x1: f32,
        z0: f32,
        z1: f32,
        k: f32,
        flip_normals: bool,
    },
    YZ {
        y0: f32,
        y1: f32,
        z0: f32,
        z1: f32,
        k: f32,
        flip_normals: bool,
    },
}

const FLIP_SIGN: [f32; 2] = [1.0, -1.0];

impl Rect {
    #[inline]
    pub fn new_xy(x0: f32, x1: f32, y0: f32, y1: f32, k: f32, flip_normals: bool) -> Rect {
        Rect::XY {
            x0,
            x1,
            y0,
            y1,
            k,
            flip_normals,
        }
    }

    #[inline]
    pub fn new_xz(x0: f32, x1: f32, z0: f32, z1: f32, k: f32, flip_normals: bool) -> Rect {
        Rect::XZ {
            x0,
            x1,
            z0,
            z1,
            k,
            flip_normals,
        }
    }

    #[inline]
    pub fn new_yz(y0: f32, y1: f32, z0: f32, z1: f32, k: f32, flip_normals: bool) -> Rect {
        Rect::YZ {
            y0,
            y1,
            z0,
            z1,
            k,
            flip_normals,
        }
    }

    #[inline]
    fn xy_ray_hit(
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        x0: f32,
        x1: f32,
        y0: f32,
        y1: f32,
        k: f32,
        flip_normals: bool,
    ) -> Option<RayHit> {
        let t = (k - ray.origin.get_z()) * ray.rcp_direction.get_z();
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.get_x() + t * ray.direction.get_x();
        let y = ray.origin.get_y() + t * ray.direction.get_y();
        if x < x0 || x > x1 || y < y0 || y > y1 {
            return None;
        }
        Some(RayHit {
            point: ray.point_at_parameter(t),
            normal: vec3(0.0, 0.0, FLIP_SIGN[flip_normals as usize]),
            t,
            u: (x - x0) / (x1 - x0),
            v: (y - y0) / (y1 - y0),
        })
    }

    #[inline]
    fn xz_ray_hit(
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        x0: f32,
        x1: f32,
        z0: f32,
        z1: f32,
        k: f32,
        flip_normals: bool,
    ) -> Option<RayHit> {
        let t = (k - ray.origin.get_y()) * ray.rcp_direction.get_y();
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.get_x() + t * ray.direction.get_x();
        let z = ray.origin.get_z() + t * ray.direction.get_z();
        if x < x0 || x > x1 || z < z0 || z > z1 {
            return None;
        }
        Some(RayHit {
            point: ray.point_at_parameter(t),
            normal: vec3(0.0, FLIP_SIGN[flip_normals as usize], 0.0),
            t,
            u: (x - x0) / (x1 - x0),
            v: (z - z0) / (z1 - z0),
        })
    }

    #[inline]
    fn yz_ray_hit(
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        y0: f32,
        y1: f32,
        z0: f32,
        z1: f32,
        k: f32,
        flip_normals: bool,
    ) -> Option<RayHit> {
        let t = (k - ray.origin.get_x()) * ray.rcp_direction.get_x();
        if t < t_min || t > t_max {
            return None;
        }
        let y = ray.origin.get_y() + t * ray.direction.get_y();
        let z = ray.origin.get_z() + t * ray.direction.get_z();
        if y < y0 || y > y1 || z < z0 || z > z1 {
            return None;
        }
        Some(RayHit {
            point: ray.point_at_parameter(t),
            normal: vec3(FLIP_SIGN[flip_normals as usize], 0.0, 0.0),
            t,
            u: (y - y0) / (y1 - y0),
            v: (z - z0) / (z1 - z0),
        })
    }

    #[inline]
    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
        match *self {
            Rect::XY {
                x0,
                x1,
                y0,
                y1,
                k,
                flip_normals,
            } => Rect::xy_ray_hit(ray, t_min, t_max, x0, x1, y0, y1, k, flip_normals),
            Rect::XZ {
                x0,
                x1,
                z0,
                z1,
                k,
                flip_normals,
            } => Rect::xz_ray_hit(ray, t_min, t_max, x0, x1, z0, z1, k, flip_normals),
            Rect::YZ {
                y0,
                y1,
                z0,
                z1,
                k,
                flip_normals,
            } => Rect::yz_ray_hit(ray, t_min, t_max, y0, y1, z0, z1, k, flip_normals),
        }
    }

    #[inline]
    pub fn bounding_box(&self) -> AABB {
        match *self {
            Rect::XY {
                x0,
                x1,
                y0,
                y1,
                k,
                flip_normals: _,
            } => AABB {
                min: vec3(x0, y0, k - 0.0001),
                max: vec3(x1, y1, k + 0.0001),
            },
            Rect::XZ {
                x0,
                x1,
                z0,
                z1,
                k,
                flip_normals: _,
            } => AABB {
                min: vec3(x0, k - 0.0001, z0),
                max: vec3(x1, k + 0.0001, z1),
            },
            Rect::YZ {
                y0,
                y1,
                z0,
                z1,
                k,
                flip_normals: _,
            } => AABB {
                min: vec3(k - 0.0001, y0, z0),
                max: vec3(k - 0.0001, y1, z1),
            },
        }
    }
}
