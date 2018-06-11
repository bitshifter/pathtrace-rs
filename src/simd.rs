#![allow(dead_code)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[macro_export]
macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

pub union F32x4 {
    pub simd: __m128,
    pub array: [f32; 4],
}

pub union I32x4 {
    pub simd: __m128i,
    pub array: [i32; 4],
}

pub fn simd_bits() -> usize {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            return unsafe { simd_bits_avx2() };
        }
        if is_x86_feature_detected!("sse2") {
            return unsafe { simd_bits_sse2() };
        }
    }
    32
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "sse2"))]
pub unsafe fn simd_bits_sse2() -> usize {
    128
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "avx2"))]
pub unsafe fn simd_bits_avx2() -> usize {
    256
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "sse2"))]
pub unsafe fn blend_i32_sse2(lhs: __m128i, rhs: __m128i, cond: __m128) -> __m128i {
    let d = _mm_srai_epi32(_mm_castps_si128(cond), 31);
    _mm_or_si128(_mm_and_si128(d, rhs), _mm_andnot_si128(d, lhs))
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "sse2"))]
pub unsafe fn hmin_sse2(v: __m128) -> f32 {
    let v = _mm_min_ps(v, _mm_shuffle_ps(v, v, _mm_shuffle!(0, 0, 3, 2)));
    let v = _mm_min_ps(v, _mm_shuffle_ps(v, v, _mm_shuffle!(0, 0, 0, 1)));
    _mm_cvtss_f32(v)
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "sse2"))]
pub unsafe fn blend_f32_sse2(lhs: __m128, rhs: __m128, cond: __m128) -> __m128 {
    let d = _mm_castsi128_ps(_mm_srai_epi32(_mm_castps_si128(cond), 31));
    _mm_or_ps(_mm_and_ps(d, rhs), _mm_andnot_ps(d, lhs))
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "sse2"))]
pub unsafe fn dot3_sse2(
    x0: __m128,
    x1: __m128,
    y0: __m128,
    y1: __m128,
    z0: __m128,
    z1: __m128,
) -> __m128 {
    let mut dot = _mm_mul_ps(x0, x1);
    dot = _mm_add_ps(dot, _mm_mul_ps(y0, y1));
    dot = _mm_add_ps(dot, _mm_mul_ps(z0, z1));
    dot
}
