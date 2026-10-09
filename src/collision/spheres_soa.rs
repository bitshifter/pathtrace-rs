#![allow(dead_code)]
use crate::{
    collision::{AABB, Hitable, Ray, RayHit},
    material::Material,
    math::align_to,
    simd::*,
};
use glam::{Vec3, Vec3A, vec3};

#[cfg(feature = "fearless_simd")]
use fearless_simd::{Level, dispatch, prelude::*};
#[cfg(feature = "fearless_simd")]
use fearless_simd_macros::simd;

#[derive(Debug)]
pub struct SpheresSoA<'a> {
    bounds: AABB,
    feature: TargetFeature,
    centre_x: Vec<f32>,
    centre_y: Vec<f32>,
    centre_z: Vec<f32>,
    radius_sq: Vec<f32>,
    radius: Vec<f32>,
    material: Vec<Option<&'a Material<'a>>>,
    len: usize,
    num_spheres: usize,
}

impl<'a> SpheresSoA<'a> {
    pub fn new(hitables: &[Hitable<'a>]) -> SpheresSoA<'a> {
        let feature = TargetFeature::detect();
        feature.print_version();

        let mut bounds = AABB::invalid();
        let chunk_size = TargetFeature::detect().get_bits() / 32;
        let num_spheres = hitables.len();
        let len = align_to(num_spheres, chunk_size);
        let mut centre_x = Vec::with_capacity(len);
        let mut centre_y = Vec::with_capacity(len);
        let mut centre_z = Vec::with_capacity(len);
        let mut radius_sq = Vec::with_capacity(len);
        let mut radius = Vec::with_capacity(len);
        let mut material: Vec<Option<&'a Material>> = Vec::with_capacity(len);
        for hitable in hitables {
            if let Hitable::Sphere(sphere, mat) = hitable {
                bounds.add_assign(&sphere.bounding_box());
                centre_x.push(sphere.centre().x);
                centre_y.push(sphere.centre().y);
                centre_z.push(sphere.centre().z);
                radius_sq.push(sphere.radius() * sphere.radius());
                radius.push(sphere.radius());
                material.push(Some(mat));
            } else {
                panic!("Expected Hitable::Sphere, got {:?}", hitable);
            }
        }
        let padding = len - num_spheres;
        for _ in 0..padding {
            centre_x.push(f32::MAX);
            centre_y.push(f32::MAX);
            centre_z.push(f32::MAX);
            radius_sq.push(0.0);
            radius.push(1.0);
            material.push(None);
        }
        SpheresSoA {
            bounds,
            feature,
            centre_x,
            centre_y,
            centre_z,
            radius_sq,
            radius,
            material,
            len,
            num_spheres,
        }
    }

    #[inline]
    pub fn in_bounds(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        self.bounds.ray_hit(ray, t_min, t_max)
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

    pub fn ray_hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material<'_>)> {
        match self.feature {
            TargetFeature::AVX2 => unsafe { self.hit_avx2(ray, t_min, t_max) },
            TargetFeature::SSE4_1 => unsafe { self.hit_sse4_1(ray, t_min, t_max) },
            #[cfg(feature = "fearless_simd")]
            TargetFeature::PortableSimd => self.hit_portable_simd(ray, t_min, t_max),
            #[cfg(feature = "fearless_simd")]
            TargetFeature::AutoVectorize => self.hit_auto_vectorize(ray, t_min, t_max),
            TargetFeature::FallBack => self.hit_scalar(ray, t_min, t_max),
        }
    }

    pub fn hit_scalar(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(RayHit, &Material<'_>)> {
        let a = ray.direction.dot(ray.direction);
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
            let centre = vec3(*centre_x, *centre_y, *centre_z);
            let oc = ray.origin - centre;
            let b = oc.dot(ray.direction);
            let c = oc.dot(oc) - radius_sq;
            let discriminant = b * b - a * c;
            if discriminant > 0.0 {
                let discriminant_sqrt = discriminant.sqrt();
                let t_near = (-b - discriminant_sqrt) / a;
                let t_far = (-b + discriminant_sqrt) / a;
                let t = if t_near > t_min { t_near } else { t_far };
                if t > t_min && t < hit_t {
                    hit_t = t;
                    hit_index = index;
                }
            }
        }
        if hit_index < self.len {
            let point = ray.point_at_parameter(hit_t);
            let centre = vec3(
                self.centre_x[hit_index],
                self.centre_y[hit_index],
                self.centre_z[hit_index],
            );
            let normal = (point - centre) / self.radius[hit_index];
            let material = self.material[hit_index].unwrap();
            let (u, v) = material.get_sphere_uv(normal);
            Some((
                RayHit {
                    point,
                    normal,
                    t: hit_t,
                    u,
                    v,
                },
                material,
            ))
        } else {
            None
        }
    }

    /// Build the `RayHit` for a winning sphere index, shared by the fearless_simd paths.
    #[cfg(feature = "fearless_simd")]
    #[inline]
    fn hit_record(
        &self,
        ray: &Ray,
        hit_t: f32,
        hit_index: usize,
    ) -> Option<(RayHit, &Material<'_>)> {
        if hit_index < self.len {
            let point = ray.point_at_parameter(hit_t);
            let centre = vec3(
                self.centre_x[hit_index],
                self.centre_y[hit_index],
                self.centre_z[hit_index],
            );
            let normal = (point - centre) / self.radius[hit_index];
            let material = self.material[hit_index].unwrap();
            let (u, v) = material.get_sphere_uv(normal);
            Some((
                RayHit {
                    point,
                    normal,
                    t: hit_t,
                    u,
                    v,
                },
                material,
            ))
        } else {
            None
        }
    }

    /// Ray hit using `fearless_simd`'s `#[simd]` auto-vectorization.
    ///
    /// The body is ordinary scalar code; `#[simd]` compiles a copy for each available SIMD
    /// level with that level's target features enabled, and `dispatch!` runs the best one.
    #[cfg(feature = "fearless_simd")]
    pub fn hit_auto_vectorize(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material<'_>)> {
        let level = Level::new();
        dispatch!(level, simd => self.hit_auto_vectorize_impl(simd, ray, t_min, t_max))
    }

    #[cfg(feature = "fearless_simd")]
    #[simd]
    fn hit_auto_vectorize_impl<S: Simd>(
        &self,
        _: S,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material<'_>)> {
        let a = ray.direction.dot(ray.direction);
        let mut hit_t = t_max;
        let mut hit_index = self.len;
        for index in 0..self.len {
            let centre = vec3(
                self.centre_x[index],
                self.centre_y[index],
                self.centre_z[index],
            );
            let oc = ray.origin - centre;
            let b = oc.dot(ray.direction);
            let c = oc.dot(oc) - self.radius_sq[index];
            let discr = b * b - a * c;
            // `discr.sqrt()` is NaN for a miss, so the comparisons below stay branch-free.
            let discr_sqrt = discr.sqrt();
            let t_near = (-b - discr_sqrt) / a;
            let t_far = (-b + discr_sqrt) / a;
            let t = if t_near > t_min { t_near } else { t_far };
            let is_hit = (discr > 0.0) & (t > t_min) & (t < hit_t);
            hit_index = if is_hit { index } else { hit_index };
            hit_t = if is_hit { t } else { hit_t };
        }
        self.hit_record(ray, hit_t, hit_index)
    }

    /// Ray hit using `fearless_simd`'s portable SIMD vector types.
    ///
    /// Mirrors `hit_sse4_1`/`hit_avx2`, but generic over the `Simd` level and using the
    /// CPU's native `f32` width selected by `dispatch!`.
    #[cfg(feature = "fearless_simd")]
    pub fn hit_portable_simd(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material<'_>)> {
        let level = Level::new();
        dispatch!(level, simd => self.hit_portable_simd_impl(simd, ray, t_min, t_max))
    }

    #[cfg(feature = "fearless_simd")]
    #[simd]
    fn hit_portable_simd_impl<S: Simd>(
        &self,
        simd: S,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material<'_>)> {
        // `self.len` is padded to a multiple of the native width, see `SpheresSoA::new`.
        const LANE_OFFSETS: [u32; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let num_lanes = S::f32s::LEN;
        assert!(num_lanes <= LANE_OFFSETS.len());

        let a = S::f32s::splat(simd, ray.direction.dot(ray.direction));
        let ro_x = S::f32s::splat(simd, ray.origin.x);
        let ro_y = S::f32s::splat(simd, ray.origin.y);
        let ro_z = S::f32s::splat(simd, ray.origin.z);
        let rd_x = S::f32s::splat(simd, ray.direction.x);
        let rd_y = S::f32s::splat(simd, ray.direction.y);
        let rd_z = S::f32s::splat(simd, ray.direction.z);
        let t_min_v = S::f32s::splat(simd, t_min);
        let mut hit_t = S::f32s::splat(simd, t_max);
        let mut hit_index = S::u32s::splat(simd, self.len as u32);
        let lane_offsets = S::u32s::from_slice(simd, &LANE_OFFSETS[..num_lanes]);

        for chunk_index in (0..self.len).step_by(num_lanes) {
            let c_x =
                S::f32s::from_slice(simd, &self.centre_x[chunk_index..chunk_index + num_lanes]);
            let c_y =
                S::f32s::from_slice(simd, &self.centre_y[chunk_index..chunk_index + num_lanes]);
            let c_z =
                S::f32s::from_slice(simd, &self.centre_z[chunk_index..chunk_index + num_lanes]);
            let r_sq =
                S::f32s::from_slice(simd, &self.radius_sq[chunk_index..chunk_index + num_lanes]);

            let oc_x = ro_x - c_x;
            let oc_y = ro_y - c_y;
            let oc_z = ro_z - c_z;
            let b = oc_x * rd_x + oc_y * rd_y + oc_z * rd_z;
            let c = oc_x * oc_x + oc_y * oc_y + oc_z * oc_z - r_sq;
            let discr = b * b - a * c;
            let pos_discr = discr.simd_gt(0.0);
            if pos_discr.any_true() {
                let discr_sqrt = discr.sqrt();
                let t0 = (-b - discr_sqrt) / a;
                let t1 = (-b + discr_sqrt) / a;
                let t = t0.simd_gt(t_min_v).select(t0, t1);
                let mask = pos_discr & t.simd_gt(t_min_v) & t.simd_lt(hit_t);
                let index = S::u32s::splat(simd, chunk_index as u32) + lane_offsets;
                hit_index = mask.select(index, hit_index);
                hit_t = mask.select(t, hit_t);
            }
        }

        let min_hit_t = hit_t.reduce_min();
        if min_hit_t < t_max {
            let min_mask = hit_t.simd_eq(min_hit_t).to_bitmask();
            if min_mask != 0 {
                // Lowest set bit is the lowest lane, matching the scalar tie-break.
                let lane = min_mask.trailing_zeros() as usize;
                let hit_index = hit_index.as_slice()[lane] as usize;
                return self.hit_record(ray, min_hit_t, hit_index);
            }
        }
        None
    }

    #[cfg_attr(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature(enable = "sse4.1")
    )]
    pub unsafe fn hit_sse4_1(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material<'_>)> {
        unsafe {
            #[cfg(target_arch = "x86")]
            use std::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use std::arch::x86_64::*;
            const NUM_LANES: usize = 4;
            let t_min = _mm_set_ps1(t_min);
            let mut hit_t = _mm_set_ps1(t_max);
            let mut hit_index = _mm_set_epi32(-1, -1, -1, -1);
            // load ray origin
            let ro = Vec3A::from(ray.origin).into();
            let ro_x = _mm_shuffle_ps(ro, ro, 0b00_00_00_00);
            let ro_y = _mm_shuffle_ps(ro, ro, 0b01_01_01_01);
            let ro_z = _mm_shuffle_ps(ro, ro, 0b10_10_10_10);
            // load ray direction
            let rd = Vec3A::from(ray.direction).into();
            let rd_x = _mm_shuffle_ps(rd, rd, 0b00_00_00_00);
            let rd_y = _mm_shuffle_ps(rd, rd, 0b01_01_01_01);
            let rd_z = _mm_shuffle_ps(rd, rd, 0b10_10_10_10);
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
                // ray direction dot product
                let a = _mm_set_ps1(ray.direction.dot(ray.direction));
                // let oc = ray.origin - centre
                let oc_x = _mm_sub_ps(ro_x, c_x);
                let oc_y = _mm_sub_ps(ro_y, c_y);
                let oc_z = _mm_sub_ps(ro_z, c_z);
                // let b = dot(oc, ray.direction);
                let b = dot3_sse2(oc_x, rd_x, oc_y, rd_y, oc_z, rd_z);
                // let c = dot(oc, oc) - radius_sq;
                let c = _mm_sub_ps(dot3_sse2(oc_x, oc_x, oc_y, oc_y, oc_z, oc_z), r_sq);
                // let discriminant = b * b - a * c;
                let discr = _mm_sub_ps(_mm_mul_ps(b, b), _mm_mul_ps(a, c));
                // if discr > 0.0
                let pos_discr = _mm_cmpgt_ps(discr, _mm_set_ps1(0.0));
                if _mm_movemask_ps(pos_discr) != 0 {
                    // let discr_sqrt = discr.sqrt();
                    let discr_sqrt = _mm_sqrt_ps(discr);
                    // let neg_b = -b;
                    let neg_b = _mm_sub_ps(_mm_set_ps1(0.0), b);
                    // let t0 = (-b - discr_sqrt) / a;
                    let t0 = _mm_div_ps(_mm_sub_ps(neg_b, discr_sqrt), a);
                    // let t1 = (-b + discr_sqrt) / a;
                    let t1 = _mm_div_ps(_mm_add_ps(neg_b, discr_sqrt), a);
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
                    let hit_t_lane = cttz_4bits_nonzero(min_mask as u32) as usize;
                    debug_assert!(hit_t_lane < NUM_LANES);

                    let hit_index_array = I32x4 { simd: hit_index }.array;
                    let hit_t_array = F32x4 { simd: hit_t }.array;

                    let hit_index_scalar = *hit_index_array.get_unchecked(hit_t_lane) as usize;
                    debug_assert!(hit_index_scalar < self.len);
                    let hit_t_scalar = *hit_t_array.get_unchecked(hit_t_lane);

                    let point = ray.point_at_parameter(hit_t_scalar);
                    let centre = vec3(
                        *self.centre_x.get_unchecked(hit_index_scalar),
                        *self.centre_y.get_unchecked(hit_index_scalar),
                        *self.centre_z.get_unchecked(hit_index_scalar),
                    );
                    let normal =
                        (point - centre) / *self.radius.get_unchecked(hit_index_scalar);
                    let material = self.material.get_unchecked(hit_index_scalar).unwrap();
                    let (u, v) = material.get_sphere_uv(normal);
                    return Some((
                        RayHit {
                            point,
                            normal,
                            t: hit_t_scalar,
                            u,
                            v,
                        },
                        material,
                    ));
                }
            }
            None
        }
    }

    #[cfg_attr(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature(enable = "avx2")
    )]
    pub unsafe fn hit_avx2(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(RayHit, &Material<'_>)> {
        unsafe {
            #[cfg(target_arch = "x86")]
            use std::arch::x86::*;
            #[cfg(target_arch = "x86_64")]
            use std::arch::x86_64::*;
            const NUM_LANES: usize = 8;
            let t_min = _mm256_set1_ps(t_min);
            let mut hit_t = _mm256_set1_ps(t_max);
            let mut hit_index = _mm256_set1_epi32(-1);
            // load ray origin
            let ro = Vec3A::from(ray.origin).into();
            let ro_x = _mm_shuffle_ps(ro, ro, 0b00_00_00_00);
            let ro_y = _mm_shuffle_ps(ro, ro, 0b01_01_01_01);
            let ro_z = _mm_shuffle_ps(ro, ro, 0b10_10_10_10);
            let ro_x = _mm256_set_m128(ro_x, ro_x);
            let ro_y = _mm256_set_m128(ro_y, ro_y);
            let ro_z = _mm256_set_m128(ro_z, ro_z);
            // load ray direction
            let rd = Vec3A::from(ray.direction).into();
            let rd_x = _mm_shuffle_ps(rd, rd, 0b00_00_00_00);
            let rd_y = _mm_shuffle_ps(rd, rd, 0b01_01_01_01);
            let rd_z = _mm_shuffle_ps(rd, rd, 0b10_10_10_10);
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
                // ray direction dot product
                let a = _mm256_set1_ps(ray.direction.dot(ray.direction));
                // let oc = ray.origin - centre
                let oc_x = _mm256_sub_ps(ro_x, c_x);
                let oc_y = _mm256_sub_ps(ro_y, c_y);
                let oc_z = _mm256_sub_ps(ro_z, c_z);
                // let b = dot(oc, ray.direction);
                let b = dot3_avx2(oc_x, rd_x, oc_y, rd_y, oc_z, rd_z);
                // let c = dot(oc, oc) - radius_sq;
                let c = _mm256_sub_ps(dot3_avx2(oc_x, oc_x, oc_y, oc_y, oc_z, oc_z), r_sq);
                // let discriminant = b * b - a * c;
                let discr = _mm256_sub_ps(_mm256_mul_ps(b, b), _mm256_mul_ps(a, c));
                // if discr > 0.0
                let pos_discr = _mm256_cmp_ps(discr, _mm256_set1_ps(0.0), _CMP_GT_OQ);
                if _mm256_movemask_ps(pos_discr) != 0 {
                    // let discr_sqrt = discr.sqrt();
                    let discr_sqrt = _mm256_sqrt_ps(discr);
                    // let neg_b = -b;
                    let neg_b = _mm256_sub_ps(_mm256_set1_ps(0.0), b);
                    // let t0 = (-b - discr_sqrt) / a;
                    let t0 = _mm256_div_ps(_mm256_sub_ps(neg_b, discr_sqrt), a);
                    // let t1 = (-b + discr_sqrt) / a;
                    let t1 = _mm256_div_ps(_mm256_add_ps(neg_b, discr_sqrt), a);
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
                    let hit_t_lane = cttz_8bits_nonzero(min_mask as u32) as usize;
                    debug_assert!(hit_t_lane < NUM_LANES);

                    let hit_index_array = I32x8 { simd: hit_index }.array;
                    let hit_t_array = F32x8 { simd: hit_t }.array;

                    let hit_index_scalar = *hit_index_array.get_unchecked(hit_t_lane) as usize;
                    debug_assert!(hit_index_scalar < self.len);
                    let hit_t_scalar = *hit_t_array.get_unchecked(hit_t_lane);

                    let point = ray.point_at_parameter(hit_t_scalar);
                    let centre = vec3(
                        *self.centre_x.get_unchecked(hit_index_scalar),
                        *self.centre_y.get_unchecked(hit_index_scalar),
                        *self.centre_z.get_unchecked(hit_index_scalar),
                    );
                    let normal =
                        (point - centre) / *self.radius.get_unchecked(hit_index_scalar);
                    let material = self.material.get_unchecked(hit_index_scalar).unwrap();
                    let (u, v) = material.get_sphere_uv(normal);
                    return Some((
                        RayHit {
                            point,
                            normal,
                            t: hit_t_scalar,
                            u,
                            v,
                        },
                        material,
                    ));
                }
            }
            None
        }
    }
}

#[inline]
fn cttz_8bits_nonzero(x: u32) -> u32 {
    // cttz on first 8 bits - 0 not expected
    #[cfg(feature = "core_intrinsics")]
    {
        use std::intrinsics::cttz_nonzero;
        unsafe { cttz_nonzero(x) }
    }
    #[cfg(not(feature = "core_intrinsics"))]
    {
        let mut x = x;
        let mut n = 0;
        if (x & 0x0000000F) == 0 {
            n += 4;
            x >>= 4;
        }
        if (x & 0x00000003) == 0 {
            n += 2;
            x >>= 2;
        }
        if (x & 0x00000001) == 0 {
            n += 1;
        }
        n
    }
}

#[inline]
fn cttz_4bits_nonzero(x: u32) -> u32 {
    // cttz on first 4 bits - 0 not expected
    #[cfg(feature = "core_intrinsics")]
    {
        use std::intrinsics::cttz_nonzero;
        return unsafe { cttz_nonzero(x) };
    }
    #[cfg(not(feature = "core_intrinsics"))]
    {
        let mut x = x;
        let mut n = 0;
        if (x & 0x00000003) == 0 {
            n += 2;
            x >>= 2;
        }
        if (x & 0x00000001) == 0 {
            n += 1;
        }
        n
    }
}

#[cfg(all(test, feature = "fearless_simd"))]
mod fearless_tests {
    use super::*;
    use crate::{
        collision::Sphere,
        material::metal,
        scene::{MAX_T, MIN_T},
        storage::Storage,
    };
    use glam::vec3;
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro256Plus;

    /// Deterministic pseudo-random float in `[-1.0, 1.0]`.
    struct Lcg(u32);

    impl Lcg {
        fn next(&mut self) -> f32 {
            self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (self.0 >> 8) as f32 / (1 << 24) as f32 * 2.0 - 1.0
        }
    }

    fn assert_same_hit(
        ray: &Ray,
        expected: Option<(RayHit, &Material<'_>)>,
        actual: Option<(RayHit, &Material<'_>)>,
    ) {
        match (expected, actual) {
            (None, None) => {}
            (Some((e, e_mat)), Some((a, a_mat))) => {
                assert!((a.t - e.t).abs() <= 1e-4, "t mismatch: {} vs {}", e.t, a.t);
                assert!(
                    (a.point - e.point).length() <= 1e-3,
                    "point mismatch: {:?} vs {:?}",
                    e.point,
                    a.point
                );
                assert!(
                    (a.normal - e.normal).length() <= 1e-3,
                    "normal mismatch: {:?} vs {:?}",
                    e.normal,
                    a.normal
                );
                assert!(
                    (a.u - e.u).abs() <= 1e-3 && (a.v - e.v).abs() <= 1e-3,
                    "uv mismatch"
                );
                assert!(std::ptr::eq(e_mat, a_mat), "material mismatch");
            }
            (e, a) => panic!(
                "ray {:?} -> {:?}: scalar hit={} but fearless hit={}",
                ray.origin,
                ray.direction,
                e.is_some(),
                a.is_some()
            ),
        }
    }

    #[test]
    fn fearless_hits_match_scalar() {
        let mut rng = Xoshiro256Plus::seed_from_u64(0);
        let storage = Storage::new(&mut rng);
        let material = storage.alloc_material(metal(vec3(0.8, 0.2, 0.1), 0.0));

        // 40 spheres gives several native-width chunks plus padding for 4/8/16 lanes.
        let mut lcg = Lcg(0x1234_5678);
        let mut hitables = Vec::new();
        for i in 0..40 {
            let centre = vec3(lcg.next() * 3.0, lcg.next() * 3.0, -1.0 - i as f32 * 0.4);
            let radius = 0.2 + lcg.next().abs() * 0.8;
            let sphere = storage.alloc_sphere(Sphere::new(centre, radius));
            hitables.push(Hitable::Sphere(sphere, material));
        }
        let spheres = SpheresSoA::new(&hitables);

        let origins = [
            vec3(0.0, 0.0, 0.0),
            vec3(0.3, -0.4, 1.5),
            vec3(-1.0, 0.8, 0.5),
            vec3(2.5, 2.5, 2.5),
            // Inside the first sphere.
            vec3(0.0, 0.0, -1.0),
        ];
        for origin in origins {
            for _ in 0..256 {
                let direction = vec3(lcg.next(), lcg.next(), -lcg.next().abs() - 0.05).normalize();
                let ray = Ray::new(origin, direction, 0.0);
                let expected = spheres.hit_scalar(&ray, MIN_T, MAX_T);
                assert_same_hit(
                    &ray,
                    expected,
                    spheres.hit_auto_vectorize(&ray, MIN_T, MAX_T),
                );
                assert_same_hit(
                    &ray,
                    expected,
                    spheres.hit_portable_simd(&ray, MIN_T, MAX_T),
                );
            }
        }
    }

    #[test]
    fn fearless_hits_match_scalar_on_preset() {
        use crate::params::{Params, SoaMode};

        const PARAMS: Params = Params {
            width: 200,
            height: 100,
            samples: 10,
            max_depth: 10,
            random_seed: false,
            use_bvh: false,
            soa: SoaMode::Auto,
        };
        let mut rng = PARAMS.new_rng();
        let storage = Storage::new(&mut rng);
        let (hitables, camera, _) = crate::presets::random_spheres(&PARAMS, &mut rng, &storage);
        assert!(hitables.iter().all(|h| matches!(h, Hitable::Sphere(..))));
        let spheres = SpheresSoA::new(&hitables);
        for i in 0..16 {
            for j in 0..16 {
                let ray = camera.get_ray(i as f32 / 16.0, j as f32 / 16.0, &mut rng);
                let expected = spheres.hit_scalar(&ray, MIN_T, MAX_T);
                assert_same_hit(
                    &ray,
                    expected,
                    spheres.hit_auto_vectorize(&ray, MIN_T, MAX_T),
                );
                assert_same_hit(
                    &ray,
                    expected,
                    spheres.hit_portable_simd(&ray, MIN_T, MAX_T),
                );
            }
        }
    }
}
