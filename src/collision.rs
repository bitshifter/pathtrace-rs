use material::{Material, MaterialKind};
use math::align_to;
use simd::*;
use std::f32;
use vmath::{vec3, Vec3};

// TODO: how do I import this from mod simd
macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

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
    len: usize,
    num_spheres: usize,
}

impl SpheresSoA {
    pub fn new(spheres: &[Sphere]) -> SpheresSoA {
        // HACK: make sure there's enough entries for SIMD
        // TODO: conditionally compile this
        let chunk_size = simd_bits() / 32;
        let num_spheres = spheres.len();
        let len = align_to(num_spheres, chunk_size);
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
            centre_x,
            centre_y,
            centre_z,
            radius_sq,
            radius_inv,
            len,
            num_spheres,
        }
    }

    pub fn centre(&self, index: u32) -> Vec3 {
        let index = index as usize;
        assert!(index < self.len);
        unsafe {
            vec3(
                *self.centre_x.get_unchecked(index),
                *self.centre_y.get_unchecked(index),
                *self.centre_z.get_unchecked(index),
            )
        }
    }

    pub fn radius_sq(&self, index: u32) -> f32 {
        self.radius_sq[index as usize]
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, u32)> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { self.hit_avx2(ray, t_min, t_max) };
            }
            if is_x86_feature_detected!("sse4.1") {
                return unsafe { self.hit_sse4_1(ray, t_min, t_max) };
            }
        }

        self.hit_scalar(ray, t_min, t_max)
    }

    fn hit_scalar(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, u32)> {
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
            let co = vec3(*centre_x, *centre_y, *centre_z) - ray.origin;
            let nb = co.dot(ray.direction);
            let c = co.dot(co) - radius_sq;
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
            let normal = (point
                - vec3(
                    self.centre_x[hit_index],
                    self.centre_y[hit_index],
                    self.centre_z[hit_index],
                )) * self.radius_inv[hit_index];
            Some((RayHit { point, normal }, hit_index as u32))
        } else {
            None
        }
    }

    #[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "sse4.1"))]
    unsafe fn hit_sse4_1(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, u32)> {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        use std::intrinsics::cttz;
        const NUM_LANES: usize = 4;
        let t_min = _mm_set_ps1(t_min);
        let mut hit_t = _mm_set_ps1(t_max);
        let mut hit_index = _mm_set_epi32(-1, -1, -1, -1);
        // load ray origin
        let ro = ray.origin.unwrap();
        let ro_x = _mm_shuffle_ps(ro, ro, _mm_shuffle!(0, 0, 0, 0));
        let ro_y = _mm_shuffle_ps(ro, ro, _mm_shuffle!(1, 1, 1, 1));
        let ro_z = _mm_shuffle_ps(ro, ro, _mm_shuffle!(2, 2, 2, 2));
        // load ray direction
        let rd = ray.direction.unwrap();
        let rd_x = _mm_shuffle_ps(rd, rd, _mm_shuffle!(0, 0, 0, 0));
        let rd_y = _mm_shuffle_ps(rd, rd, _mm_shuffle!(1, 1, 1, 1));
        let rd_z = _mm_shuffle_ps(rd, rd, _mm_shuffle!(2, 2, 2, 2));
        // current indices being processed (little endian ordering)
        let mut index = _mm_set_epi32(3, 2, 1, 0);
        // loop over 4 spheres at a time
        let num_chunks = self.len >> 2;
        for chunk_index in (0..num_chunks).map(|i| i << 2) {
            // load sphere centres
            let c_x = _mm_loadu_ps(self.centre_x.get_unchecked(chunk_index));
            let c_y = _mm_loadu_ps(self.centre_y.get_unchecked(chunk_index));
            let c_z = _mm_loadu_ps(self.centre_z.get_unchecked(chunk_index));
            // load radius_sq
            let r_sq = _mm_loadu_ps(self.radius_sq.get_unchecked(chunk_index));
            // let co = centre - ray.origin
            let co_x = _mm_sub_ps(c_x, ro_x);
            let co_y = _mm_sub_ps(c_y, ro_y);
            let co_z = _mm_sub_ps(c_z, ro_z);
            // let nb = dot(co, ray.direction);
            let nb = dot3_sse2(co_x, rd_x, co_y, rd_y, co_z, rd_z);
            // let c = dot(co, co) - radius_sq;
            let c = _mm_sub_ps(dot3_sse2(co_x, co_x, co_y, co_y, co_z, co_z), r_sq);
            // let discriminant = nb * nb - c;
            let discr = _mm_sub_ps(_mm_mul_ps(nb, nb), c);
            // if discr > 0.0
            let pos_discr = _mm_cmpgt_ps(discr, _mm_set_ps1(0.0));
            if _mm_movemask_ps(pos_discr) != 0 {
                // let discr_sqrt = discr.sqrt();
                let discr_sqrt = _mm_sqrt_ps(discr);
                // let t0 = nb - discr_sqrt;
                let t0 = _mm_sub_ps(nb, discr_sqrt);
                // let t1 = nb + discr_sqrt;
                let t1 = _mm_add_ps(nb, discr_sqrt);
                // let t = if t0 > t_min { t0 } else { t1 };
                let t = _mm_blendv_ps(t1, t0, _mm_cmpgt_ps(t0, t_min));
                // from rygs opts
                // bool4 msk = discrPos & (t > tMin4) & (t < hitT);
                let mask = _mm_and_ps(
                    pos_discr,
                    _mm_and_ps(_mm_cmpgt_ps(t, t_min), _mm_cmplt_ps(t, hit_t)),
                );
                // hit_index = mask ? index : hit_index;
                hit_index = _mm_blendv_epi8(hit_index, index, _mm_castps_si128(mask));
                // hit_t = mask ? t : hit_t;
                hit_t = _mm_blendv_ps(hit_t, t, mask);
            }
            // increment indices
            index = _mm_add_epi32(index, _mm_set1_epi32(NUM_LANES as i32));
        }

        let min_hit_t = hmin_sse2(hit_t);
        if min_hit_t < t_max {
            let min_mask = _mm_movemask_ps(_mm_cmpeq_ps(hit_t, _mm_set1_ps(min_hit_t)));
            if min_mask != 0 {
                let hit_t_lane = cttz(min_mask) as usize;
                debug_assert!(hit_t_lane < NUM_LANES);

                let hit_index_array = I32x4 { simd: hit_index }.array;
                let hit_t_array = F32x4 { simd: hit_t }.array;

                let hit_index_scalar = *hit_index_array.get_unchecked(hit_t_lane) as usize;
                debug_assert!(hit_index_scalar < self.len);
                let hit_t_scalar = *hit_t_array.get_unchecked(hit_t_lane);

                let point = ray.point_at_parameter(hit_t_scalar);
                let normal = (point
                    - vec3(
                        *self.centre_x.get_unchecked(hit_index_scalar),
                        *self.centre_y.get_unchecked(hit_index_scalar),
                        *self.centre_z.get_unchecked(hit_index_scalar),
                    ))
                    * *self.radius_inv.get_unchecked(hit_index_scalar);
                return Some((RayHit { point, normal }, hit_index_scalar as u32));
            }
        }
        None
    }

    #[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "avx2"))]
    unsafe fn hit_avx2(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, u32)> {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        use std::intrinsics::cttz;
        // TODO: are these defined anywhere in std::arch?
        const NUM_LANES: usize = 8;
        let t_min = _mm256_set1_ps(t_min);
        let mut hit_t = _mm256_set1_ps(t_max);
        let mut hit_index = _mm256_set1_epi32(-1);
        // load ray origin
        let ro = ray.origin.unwrap();
        let ro_x = _mm_shuffle_ps(ro, ro, _mm_shuffle!(0, 0, 0, 0));
        let ro_y = _mm_shuffle_ps(ro, ro, _mm_shuffle!(1, 1, 1, 1));
        let ro_z = _mm_shuffle_ps(ro, ro, _mm_shuffle!(2, 2, 2, 2));
        let ro_x = _mm256_set_m128(ro_x, ro_x);
        let ro_y = _mm256_set_m128(ro_y, ro_y);
        let ro_z = _mm256_set_m128(ro_z, ro_z);
        // load ray direction
        let rd = ray.direction.unwrap();
        let rd_x = _mm_shuffle_ps(rd, rd, _mm_shuffle!(0, 0, 0, 0));
        let rd_y = _mm_shuffle_ps(rd, rd, _mm_shuffle!(1, 1, 1, 1));
        let rd_z = _mm_shuffle_ps(rd, rd, _mm_shuffle!(2, 2, 2, 2));
        let rd_x = _mm256_set_m128(rd_x, rd_x);
        let rd_y = _mm256_set_m128(rd_y, rd_y);
        let rd_z = _mm256_set_m128(rd_z, rd_z);
        // current indices being processed (little endian ordering)
        let mut index = _mm256_set_epi32(7, 6, 5, 4, 3, 2, 1, 0);
        // loop over NUM_LANES spheres at a time
        let num_chunks = self.len >> 3;
        for chunk_index in (0..num_chunks).map(|i| i << 3) {
            // load sphere centres
            let c_x = _mm256_loadu_ps(self.centre_x.get_unchecked(chunk_index));
            let c_y = _mm256_loadu_ps(self.centre_y.get_unchecked(chunk_index));
            let c_z = _mm256_loadu_ps(self.centre_z.get_unchecked(chunk_index));
            // load radius_sq
            let r_sq = _mm256_loadu_ps(self.radius_sq.get_unchecked(chunk_index));
            // let co = centre - ray.origin
            let co_x = _mm256_sub_ps(c_x, ro_x);
            let co_y = _mm256_sub_ps(c_y, ro_y);
            let co_z = _mm256_sub_ps(c_z, ro_z);
            // let nb = dot(co, ray.direction);
            let nb = dot3_avx2(co_x, rd_x, co_y, rd_y, co_z, rd_z);
            // let c = dot(co, co) - radius_sq;
            let c = _mm256_sub_ps(dot3_avx2(co_x, co_x, co_y, co_y, co_z, co_z), r_sq);
            // let discriminant = nb * nb - c;
            let discr = _mm256_sub_ps(_mm256_mul_ps(nb, nb), c);
            // if discr > 0.0
            let pos_discr = _mm256_cmp_ps(discr, _mm256_set1_ps(0.0), _CMP_GT_OQ);
            if _mm256_movemask_ps(pos_discr) != 0 {
                // let discr_sqrt = discr.sqrt();
                let discr_sqrt = _mm256_sqrt_ps(discr);
                // let t0 = nb - discr_sqrt;
                let t0 = _mm256_sub_ps(nb, discr_sqrt);
                // let t1 = nb + discr_sqrt;
                let t1 = _mm256_add_ps(nb, discr_sqrt);
                // let t = if t0 > t_min { t0 } else { t1 };
                let t = _mm256_blendv_ps(t1, t0, _mm256_cmp_ps(t0, t_min, _CMP_GT_OQ));
                // from rygs opts
                // bool4 msk = discrPos & (t > tMin4) & (t < hitT);
                let mask = _mm256_and_ps(
                    pos_discr,
                    _mm256_and_ps(
                        _mm256_cmp_ps(t, t_min, _CMP_GT_OQ),
                        _mm256_cmp_ps(t, hit_t, _CMP_LT_OQ),
                    ),
                );
                // hit_index = mask ? index : hit_index;
                hit_index = _mm256_blendv_epi8(hit_index, index, _mm256_castps_si256(mask));
                // hit_t = mask ? t : hit_t;
                hit_t = _mm256_blendv_ps(hit_t, t, mask);
            }
            // increment indices
            index = _mm256_add_epi32(index, _mm256_set1_epi32(NUM_LANES as i32));
        }

        let min_hit_t = hmin_avx2(hit_t);
        if min_hit_t < t_max {
            let min_mask =
                _mm256_movemask_ps(_mm256_cmp_ps(hit_t, _mm256_set1_ps(min_hit_t), _CMP_EQ_OQ));
            if min_mask != 0 {
                let hit_t_lane = cttz(min_mask) as usize;
                debug_assert!(hit_t_lane < NUM_LANES);

                let hit_index_array = I32x8 { simd: hit_index }.array;
                let hit_t_array = F32x8 { simd: hit_t }.array;

                let hit_index_scalar = *hit_index_array.get_unchecked(hit_t_lane) as usize;
                debug_assert!(hit_index_scalar < self.len);
                let hit_t_scalar = *hit_t_array.get_unchecked(hit_t_lane);

                let point = ray.point_at_parameter(hit_t_scalar);
                let normal = (point
                    - vec3(
                        *self.centre_x.get_unchecked(hit_index_scalar),
                        *self.centre_y.get_unchecked(hit_index_scalar),
                        *self.centre_z.get_unchecked(hit_index_scalar),
                    ))
                    * *self.radius_inv.get_unchecked(hit_index_scalar);
                return Some((RayHit { point, normal }, hit_index_scalar as u32));
            }
        }
        None
    }
}
