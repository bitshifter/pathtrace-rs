use material::Material;
use math::align_to;
use simd::{
    dot3, f32xN, i32xN, ArrayF32xN, ArrayI32xN, VECTOR_WIDTH_DWORDS_LOG2, VECTOR_WIDTH_DWORDS,
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

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub centre: Vec3,
    pub radius: f32,
}

#[inline]
pub fn sphere(centre: Vec3, radius: f32, material: Material) -> (Sphere, Material) {
    (Sphere { centre, radius }, material)
}

#[derive(Debug)]
pub struct SpheresSoA {
    centre_x: Vec<ArrayF32xN>,
    centre_y: Vec<ArrayF32xN>,
    centre_z: Vec<ArrayF32xN>,
    radius_sq: Vec<ArrayF32xN>,
    radius_inv: Vec<ArrayF32xN>,
    material: Vec<[Material; VECTOR_WIDTH_DWORDS]>,
    num_chunks: u32,
    num_spheres: u32,
}

impl SpheresSoA {
    pub fn new(sphere_materials: &[(Sphere, Material)]) -> SpheresSoA {
        let num_spheres = sphere_materials.len();
        let num_chunks = align_to(num_spheres, VECTOR_WIDTH_DWORDS) / VECTOR_WIDTH_DWORDS;
        let mut centre_x = Vec::with_capacity(num_chunks);
        let mut centre_y = Vec::with_capacity(num_chunks);
        let mut centre_z = Vec::with_capacity(num_chunks);
        let mut radius_inv = Vec::with_capacity(num_chunks);
        let mut radius_sq = Vec::with_capacity(num_chunks);
        let mut material = Vec::with_capacity(num_chunks);
        for chunk in sphere_materials.chunks(VECTOR_WIDTH_DWORDS) {
            // this is so if we have a final chunk that is smaller than the chunk size it will be
            // initialised to impossible to hit data.
            let mut chunk_x = ArrayF32xN::new([f32::MAX; VECTOR_WIDTH_DWORDS]);
            let mut chunk_y = ArrayF32xN::new([f32::MAX; VECTOR_WIDTH_DWORDS]);
            let mut chunk_z = ArrayF32xN::new([f32::MAX; VECTOR_WIDTH_DWORDS]);
            let mut chunk_rsq = ArrayF32xN::new([0.0; VECTOR_WIDTH_DWORDS]);
            let mut chunk_rinv = ArrayF32xN::new([0.0; VECTOR_WIDTH_DWORDS]);
            let mut chunk_mat = [Material::Invalid; VECTOR_WIDTH_DWORDS];
            for (index, (sphere, mat)) in chunk.iter().enumerate() {
                chunk_x.0[index] = sphere.centre.x;
                chunk_y.0[index] = sphere.centre.y;
                chunk_z.0[index] = sphere.centre.z;
                chunk_rsq.0[index] = sphere.radius * sphere.radius;
                chunk_rinv.0[index] = 1.0 / sphere.radius;
                chunk_mat[index] = *mat;
            }
            centre_x.push(chunk_x);
            centre_y.push(chunk_y);
            centre_z.push(chunk_z);
            radius_sq.push(chunk_rsq);
            radius_inv.push(chunk_rinv);
            material.push(chunk_mat);
        }
        SpheresSoA {
            num_spheres: num_spheres as u32,
            num_chunks: num_chunks as u32,
            centre_x,
            centre_y,
            centre_z,
            radius_sq,
            radius_inv,
            material,
        }
    }

    /*
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material)> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("sse4.1") {
                return unsafe { self.hit_sse4_1(ray, t_min, t_max) };
            }
            panic!("No implementation");
        }

        // self.hit_scalar(ray, t_min, t_max)
    }

    fn hit_scalar(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material)> {
        let mut hit_t = t_max;
        let mut hit_index = self.len;
        for ((((index, centre_x), centre_y), centre_z), radius_sq) in self
            .centre_x
            .iter()
            .enumerate()
            .zip(self.centre_y.iter())
            .zip(self.centre_z.iter())
            .zip(self.radius_sq.iter())
        {
            let co = vec3(
                centre_x - ray.origin.x,
                centre_y - ray.origin.y,
                centre_z - ray.origin.z,
            );
            let nb = dot(co, ray.direction);
            let c = dot(co, co) - radius_sq;
            let discriminant = nb * nb - c;
            if discriminant > 0.0 {
                let discriminant_sqrt = discriminant.sqrt();
                let mut t = nb - discriminant_sqrt;
                if t < t_min {
                    t = nb + discriminant_sqrt;
                }
                if t > t_min && t < hit_t {
                    hit_t = t;
                    hit_index = index;
                }
            }
        }
        if hit_index < self.len {
            let point = ray.point_at_parameter(hit_t);
            let normal = vec3(
                point.x - self.centre_x[hit_index],
                point.y - self.centre_y[hit_index],
                point.z - self.centre_z[hit_index],
            ) * self.radius_inv[hit_index];
            let material = &self.material[hit_index];
            Some((RayHit { point, normal }, material))
        } else {
            None
        }
    }
    */

    #[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "sse4.1"))]
    pub unsafe fn hit_sse4_1(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material)> {
        let t_min = f32xN::from(t_min);
        let mut hit_t = f32xN::from(t_max);
        let mut hit_index = i32xN::from(-1);
        // load ray origin
        let ro_x = f32xN::from(ray.origin.x);
        let ro_y = f32xN::from(ray.origin.y);
        let ro_z = f32xN::from(ray.origin.z);
        // load ray direction
        let rd_x = f32xN::from(ray.direction.x);
        let rd_y = f32xN::from(ray.direction.y);
        let rd_z = f32xN::from(ray.direction.z);
        // current indices being processed (little endian ordering)
        // TODO: generate this based on width
        let mut sphere_index = i32xN::indices();
        for (((centre_x, centre_y), centre_z), radius_sq) in self
            .centre_x
            .iter()
            .zip(self.centre_y.iter())
            .zip(self.centre_z.iter())
            .zip(self.radius_sq.iter())
        {
            // load sphere centres
            let c_x = f32xN::from(centre_x);
            let c_y = f32xN::from(centre_y);
            let c_z = f32xN::from(centre_z);
            // load radius_sq
            let r_sq = f32xN::from(radius_sq);
            // let co = centre - ray.origin
            let co_x = c_x - ro_x;
            let co_y = c_y - ro_y;
            let co_z = c_z - ro_z;
            // let nb = dot(co, ray.direction);
            let nb = dot3(co_x, rd_x, co_y, rd_y, co_z, rd_z);
            // let c = dot(co, co) - radius_sq;
            let c = dot3(co_x, co_x, co_y, co_y, co_z, co_z) - r_sq;
            // let discriminant = nb * nb - c;
            let discr = nb * nb - c;
            // if discr > 0.0
            let ptve_discr = discr.gt(f32xN::from(0.0));
            if i32::from(ptve_discr) != 0 {
                // let discr_sqrt = discr.sqrt();
                let discr_sqrt = discr.sqrt();
                // let t0 = nb - discr_sqrt;
                // let t1 = nb + discr_sqrt;
                let t0 = nb - discr_sqrt;
                let t1 = nb + discr_sqrt;
                // let t = if t0 > t_min { t0 } else { t1 };
                let t = t1.blend(t0, t0.gt(t_min));
                // from rygs opts
                // bool4 msk = discrPos & (t > tMin4) & (t < hitT);
                let mask = ptve_discr & t.gt(t_min) & t.lt(hit_t);
                // hit_index = mask ? sphere_index : hit_index;
                hit_index = hit_index.blend(sphere_index, mask);
                // hit_t = mask ? t : hit_t;
                hit_t = hit_t.blend(t, mask);
            }
            // increment indices
            sphere_index = sphere_index + i32xN::from(VECTOR_WIDTH_DWORDS as i32);
        }

        let min_hit_t = hit_t.hmin();
        if min_hit_t < t_max {
            let min_mask = i32::from(hit_t.eq(f32xN::from(min_hit_t)));
            if min_mask != 0 {
                let hit_t_lane = cttz(min_mask) as usize;

                // store hit_index and hit_t back to scalar
                // TODO: use aligned structures
                let mut hit_index_array = ArrayI32xN::new([-1; VECTOR_WIDTH_DWORDS]);
                let mut hit_t_array = ArrayF32xN::new([t_max; VECTOR_WIDTH_DWORDS]);
                hit_index_array.store(hit_index);
                hit_t_array.store(hit_t);

                debug_assert!(hit_t_lane < hit_index_array.0.len());
                debug_assert!(hit_t_lane < hit_t_array.0.len());

                let hit_index_scalar = *hit_index_array.0.get_unchecked(hit_t_lane) as usize;
                let hit_t_scalar = *hit_t_array.0.get_unchecked(hit_t_lane);

                let chunk_index = hit_index_scalar >> VECTOR_WIDTH_DWORDS_LOG2;
                let lane_index = hit_index_scalar - (chunk_index << VECTOR_WIDTH_DWORDS_LOG2);

                debug_assert!(chunk_index < self.num_spheres as usize);
                debug_assert!(lane_index < VECTOR_WIDTH_DWORDS);

                let point = ray.point_at_parameter(hit_t_scalar);
                let normal = vec3(
                    point.x
                        - self
                            .centre_x
                            .get_unchecked(chunk_index)
                            .0
                            .get_unchecked(lane_index),
                    point.y
                        - self
                            .centre_y
                            .get_unchecked(chunk_index)
                            .0
                            .get_unchecked(lane_index),
                    point.z
                        - self
                            .centre_z
                            .get_unchecked(chunk_index)
                            .0
                            .get_unchecked(lane_index),
                )
                    * *self
                        .radius_inv
                        .get_unchecked(chunk_index)
                        .0
                        .get_unchecked(lane_index);
                let material = &self
                    .material
                    .get_unchecked(chunk_index)
                    .get_unchecked(lane_index);
                return Some((RayHit { point, normal }, material));
            }
        }
        None
    }
}
