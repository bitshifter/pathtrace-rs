use material::{Material, MaterialKind};
use math::align_to;
use simd::{
    Float32xN, Int32xN, Bool32xN, VECTOR_WIDTH_DWORDS,
};
use std::f32;
use std::intrinsics::cttz;
use vmath::{vec3, Vec3};

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

#[derive(Debug)]
pub struct SpheresSoA {
    centre_x: Vec<f32>,
    centre_y: Vec<f32>,
    centre_z: Vec<f32>,
    radius_sq: Vec<f32>,
    radius_inv: Vec<f32>,
    len: u32,
    num_spheres: u32,
}

impl SpheresSoA {
    pub fn new(spheres: &[Sphere]) -> SpheresSoA {
        let num_spheres = spheres.len();
        // align to 8 for now
        let len = align_to(num_spheres, 8);
        let mut centre_x = Vec::with_capacity(len);
        let mut centre_y = Vec::with_capacity(len);
        let mut centre_z = Vec::with_capacity(len);
        let mut radius_inv = Vec::with_capacity(len);
        let mut radius_sq = Vec::with_capacity(len);
        for sphere in spheres {
            centre_x.push(sphere.centre.get_x());
            centre_y.push(sphere.centre.get_y());
            centre_z.push(sphere.centre.get_z());
            radius_sq.push(sphere.radius * sphere.radius);
            radius_inv.push(1.0 / sphere.radius);
        }
        let padding = len - num_spheres;
        for _ in 0..padding {
            centre_x.push(f32::MAX);
            centre_y.push(f32::MAX);
            centre_z.push(f32::MAX);
            radius_sq.push(0.0);
            radius_inv.push(0.0);
        }
        SpheresSoA {
            num_spheres: num_spheres as u32,
            len: len as u32,
            centre_x,
            centre_y,
            centre_z,
            radius_sq,
            radius_inv,
        }
    }

    pub fn centre(&self, sphere_index: u32) -> Vec3 {
        debug_assert!(sphere_index < self.num_spheres);
        let sphere_index = sphere_index as usize;
        unsafe {
            vec3(
                *self
                    .centre_x
                    .get_unchecked(sphere_index),
                *self
                    .centre_y
                    .get_unchecked(sphere_index),
                *self
                    .centre_z
                    .get_unchecked(sphere_index),
            )
        }
    }

    pub fn radius_sq(&self, sphere_index: u32) -> f32 {
        debug_assert!(sphere_index < self.num_spheres);
        let sphere_index = sphere_index as usize;
        unsafe {
            *self
                .radius_sq
                .get_unchecked(sphere_index)
        }
    }

    pub fn hit_simd<FI, II, BI, B: Bool32xN<BI>, F: Float32xN<FI, BI, B>, I: Int32xN<II, BI, B>>(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, u32)> {
        let t_min = F::splat(t_min);
        let mut hit_t = F::splat(t_max);
        let mut hit_index = I::splat(-1);
        // load ray origin
        let ro_x = F::from_x(ray.origin);
        let ro_y = F::from_y(ray.origin);
        let ro_z = F::from_z(ray.origin);
        // load ray direction
        let rd_x = F::from_x(ray.direction);
        let rd_y = F::from_y(ray.direction);
        let rd_z = F::from_z(ray.direction);
        // current indices being processed (little endian ordering)
        let mut sphere_index = I::indices();
        let num_lanes = F::num_lanes();
        for (((centre_x, centre_y), centre_z), radius_sq) in self
            .centre_x
            .chunks(num_lanes)
            .zip(self.centre_y.chunks(num_lanes))
            .zip(self.centre_z.chunks(num_lanes))
            .zip(self.radius_sq.chunks(num_lanes))
        {
            // load sphere centres
            let c_x = F::load_unaligned(centre_x);
            let c_y = F::load_unaligned(centre_y);
            let c_z = F::load_unaligned(centre_z);
            // load radius_sq
            let r_sq = F::load_unaligned(radius_sq);
            // let co = centre - ray.origin
            let co_x = c_x - ro_x;
            let co_y = c_y - ro_y;
            let co_z = c_z - ro_z;
            // let nb = dot(co, ray.direction);
            let nb = F::dot3(co_x, rd_x, co_y, rd_y, co_z, rd_z);
            // let c = dot(co, co) - radius_sq;
            let c = F::dot3(co_x, co_x, co_y, co_y, co_z, co_z) - r_sq;
            let discr = nb * nb - c;
            // if discr > 0.0
            let ptve_discr = discr.gt(F::splat(0.0));
            if i32::from(ptve_discr) != 0 {
                let discr_sqrt = discr.sqrt();
                let t0 = nb - discr_sqrt;
                let t1 = nb + discr_sqrt;
                // let t = if t0 > t_min { t0 } else { t1 };
                let t = F::blend(t1, t0, t0.gt(t_min));
                // from rygs opts
                let mask = ptve_discr & t.gt(t_min) & t.lt(hit_t);
                // hit_index = mask ? sphere_index : hit_index;
                hit_index = I::blend(hit_index, sphere_index, mask);
                // hit_t = mask ? t : hit_t;
                hit_t = F::blend(hit_t, t, mask);
            }
            // increment indices
            sphere_index = sphere_index + I::splat(VECTOR_WIDTH_DWORDS as i32);
        }

        let min_hit_t = hit_t.hmin();
        if min_hit_t < t_max {
            let min_mask = I::from(hit_t.eq(F::splat(min_hit_t)));
            if min_mask != 0 {
                let hit_t_lane = unsafe { cttz(min_mask) } as usize;

                // store hit_index and hit_t back to scalar
                // TODO: use aligned structures
                let mut hit_index_array = [-1i32; VECTOR_WIDTH_DWORDS];
                let mut hit_t_array = [t_max; VECTOR_WIDTH_DWORDS];
                hit_index.store_unaligned(&mut hit_index_array);
                hit_t.store_unaligned(&mut hit_t_array);

                debug_assert!(hit_t_lane < hit_index_array.len());
                debug_assert!(hit_t_lane < hit_t_array.len());

                let hit_index_scalar =
                    unsafe { *hit_index_array.get_unchecked(hit_t_lane) as usize };
                let hit_t_scalar = unsafe { *hit_t_array.get_unchecked(hit_t_lane) };

                let point = ray.point_at_parameter(hit_t_scalar);
                let normal = unsafe {
                    (point
                        - vec3(
                            *self
                                .centre_x
                                .get_unchecked(hit_index_scalar),
                            *self
                                .centre_y
                                .get_unchecked(hit_index_scalar),
                            *self
                                .centre_z
                                .get_unchecked(hit_index_scalar),
                        ))
                        * *self
                            .radius_inv
                            .get_unchecked(hit_index_scalar)
                };
                return Some((RayHit { point, normal }, hit_index_scalar as u32));
            }
        }
        None
    }
}
