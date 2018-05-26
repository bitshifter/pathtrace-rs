use material::{Material, MaterialKind};
use vmath::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[inline]
pub fn ray(origin: Vec3, direction: Vec3) -> Ray {
    Ray { origin, direction }
}

impl Ray {
    #[inline]
    #[allow(dead_code)]
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }
    #[inline]
    pub fn point_at_parameter(&self, t: f32) -> Vec3 {
        self.origin + (t * self.direction)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

// #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub centre: Vec3,
    pub radius: f32,
}

#[inline]
pub fn sphere(
    centre: Vec3,
    radius: f32,
    kind: MaterialKind,
    emissive: Option<Vec3>,
) -> (Sphere, Material) {
    (
        Sphere { centre, radius },
        Material {
            kind,
            emissive: emissive.unwrap_or(Vec3::zero()),
        },
    )
}

impl Sphere {
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<RayHit> {
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
                return Some(RayHit { t, point, normal });
            }
            let t = (-b + discriminant_sqrt) / a;
            if t < t_max && t > t_min {
                let point = ray.point_at_parameter(t);
                let normal = (point - self.centre) / self.radius;
                return Some(RayHit { t, point, normal });
            }
        }
        None
    }
}
