#![allow(dead_code)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

macro_rules! _ps_const_ty {
    ($name:ident, $field:ident, $x:expr) => {
        const $name: UnionCast = UnionCast {
            $field: [$x, $x, $x, $x],
        };
    };
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub union F32x4 {
    pub simd: __m128,
    pub array: [f32; 4],
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub union I32x4 {
    pub simd: __m128i,
    pub array: [i32; 4],
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub union F32x8 {
    pub simd: __m256,
    pub array: [f32; 8],
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub union I32x8 {
    pub simd: __m256i,
    pub array: [i32; 8],
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub union UnionCast {
    pub m128: __m128,
    pub m128i: __m128i,
    pub f32x4: [f32; 4],
    pub i32x4: [i32; 4],
    pub u32x4: [u32; 4],
}

_ps_const_ty!(PS_INV_SIGN_MASK, u32x4, !0x8000_0000);
_ps_const_ty!(PS_SIGN_MASK, u32x4, 0x8000_0000);

_ps_const_ty!(PS_1_0, f32x4, 1.0);
_ps_const_ty!(PS_0_5, f32x4, 0.5);

_ps_const_ty!(PI32_1, i32x4, 1);
_ps_const_ty!(PI32_INV_1, i32x4, !1);
_ps_const_ty!(PI32_2, i32x4, 2);
_ps_const_ty!(PI32_4, i32x4, 4);

_ps_const_ty!(PS_MINUS_CEPHES_DP1, f32x4, -0.78515625);
_ps_const_ty!(PS_MINUS_CEPHES_DP2, f32x4, -2.4187564849853515625e-4);
_ps_const_ty!(PS_MINUS_CEPHES_DP3, f32x4, -3.77489497744594108e-8);
_ps_const_ty!(PS_SINCOF_P0, f32x4, -1.9515295891E-4);
_ps_const_ty!(PS_SINCOF_P1, f32x4, 8.3321608736E-3);
_ps_const_ty!(PS_SINCOF_P2, f32x4, -1.6666654611E-1);
_ps_const_ty!(PS_COSCOF_P0, f32x4, 2.443315711809948E-005);
_ps_const_ty!(PS_COSCOF_P1, f32x4, -1.388731625493765E-003);
_ps_const_ty!(PS_COSCOF_P2, f32x4, 4.166664568298827E-002);
_ps_const_ty!(PS_CEPHES_FOPI, f32x4, 1.27323954473516); // 4 / M_PI

pub fn sinf_cosf(x: f32) -> (f32, f32) {
    // expect sse2 to be available on all x86 builds
    #[cfg(target_feature = "sse2")]
    unsafe {
        let (sinx, cosx) = sinf_cosf_sse2(_mm_set1_ps(x));
        (_mm_cvtss_f32(sinx), _mm_cvtss_f32(cosx))
    }
    #[cfg(not(target_feature = "sse2"))]
    x.sin_cos()
}

// Based on http://gruntthepeon.free.fr/ssemath/sse_mathfun.h
#[cfg(target_feature = "sse2")]
pub unsafe fn sinf_cosf_sse2(x: __m128) -> (__m128, __m128) {
    let mut sign_bit_sin = x;
    // take the absolute value
    let mut x = _mm_and_ps(x, PS_INV_SIGN_MASK.m128);
    // extract the sign bit (upper one)
    sign_bit_sin = _mm_and_ps(sign_bit_sin, PS_SIGN_MASK.m128);

    // scale by 4/Pi
    let mut y = _mm_mul_ps(x, PS_CEPHES_FOPI.m128);

    // store the integer part of y in emm2
    let mut emm2 = _mm_cvttps_epi32(y);

    // j=(j+1) & (~1) (see the cephes sources)
    emm2 = _mm_add_epi32(emm2, PI32_1.m128i);
    emm2 = _mm_and_si128(emm2, PI32_INV_1.m128i);
    y = _mm_cvtepi32_ps(emm2);

    let mut emm4 = emm2;

    /* get the swap sign flag for the sine */
    let mut emm0 = _mm_and_si128(emm2, PI32_4.m128i);
    emm0 = _mm_slli_epi32(emm0, 29);
    let swap_sign_bit_sin = _mm_castsi128_ps(emm0);

    /* get the polynom selection mask for the sine*/
    emm2 = _mm_and_si128(emm2, PI32_2.m128i);
    emm2 = _mm_cmpeq_epi32(emm2, _mm_setzero_si128());
    let poly_mask = _mm_castsi128_ps(emm2);

    /* The magic pass: "Extended precision modular arithmetic" 
     x = ((x - y * DP1) - y * DP2) - y * DP3; */
    let mut xmm1 = PS_MINUS_CEPHES_DP1.m128;
    let mut xmm2 = PS_MINUS_CEPHES_DP2.m128;
    let mut xmm3 = PS_MINUS_CEPHES_DP3.m128;
    xmm1 = _mm_mul_ps(y, xmm1);
    xmm2 = _mm_mul_ps(y, xmm2);
    xmm3 = _mm_mul_ps(y, xmm3);
    x = _mm_add_ps(x, xmm1);
    x = _mm_add_ps(x, xmm2);
    x = _mm_add_ps(x, xmm3);

    emm4 = _mm_sub_epi32(emm4, PI32_2.m128i);
    emm4 = _mm_andnot_si128(emm4, PI32_4.m128i);
    emm4 = _mm_slli_epi32(emm4, 29);
    let sign_bit_cos = _mm_castsi128_ps(emm4);

    sign_bit_sin = _mm_xor_ps(sign_bit_sin, swap_sign_bit_sin);

    // Evaluate the first polynom  (0 <= x <= Pi/4)
    let z = _mm_mul_ps(x, x);
    y = PS_COSCOF_P0.m128;

    y = _mm_mul_ps(y, z);
    y = _mm_add_ps(y, PS_COSCOF_P1.m128);
    y = _mm_mul_ps(y, z);
    y = _mm_add_ps(y, PS_COSCOF_P2.m128);
    y = _mm_mul_ps(y, z);
    y = _mm_mul_ps(y, z);
    let tmp = _mm_mul_ps(z, PS_0_5.m128);
    y = _mm_sub_ps(y, tmp);
    y = _mm_add_ps(y, PS_1_0.m128);

    // Evaluate the second polynom  (Pi/4 <= x <= 0)
    let mut y2 = PS_SINCOF_P0.m128;
    y2 = _mm_mul_ps(y2, z);
    y2 = _mm_add_ps(y2, PS_SINCOF_P1.m128);
    y2 = _mm_mul_ps(y2, z);
    y2 = _mm_add_ps(y2, PS_SINCOF_P2.m128);
    y2 = _mm_mul_ps(y2, z);
    y2 = _mm_mul_ps(y2, x);
    y2 = _mm_add_ps(y2, x);

    // select the correct result from the two polynoms
    xmm3 = poly_mask;
    let ysin2 = _mm_and_ps(xmm3, y2);
    let ysin1 = _mm_andnot_ps(xmm3, y);
    y2 = _mm_sub_ps(y2, ysin2);
    y = _mm_sub_ps(y, ysin1);

    xmm1 = _mm_add_ps(ysin1, ysin2);
    xmm2 = _mm_add_ps(y, y2);

    // update the sign
    (
        _mm_xor_ps(xmm1, sign_bit_sin),
        _mm_xor_ps(xmm2, sign_bit_cos),
    )
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
    let v = _mm_min_ps(v, _mm_shuffle_ps(v, v, 0b00_00_11_10));
    let v = _mm_min_ps(v, _mm_shuffle_ps(v, v, 0b00_00_00_01));
    _mm_cvtss_f32(v)
}

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "avx2"))]
pub unsafe fn hmin_avx2(v: __m256) -> f32 {
    let v = _mm256_min_ps(v, _mm256_permute_ps(v, 0b00_00_11_10));
    let v = _mm256_min_ps(v, _mm256_permute_ps(v, 0b00_00_00_01));
    let v = _mm256_min_ps(v, _mm256_permute2f128_ps(v, v, 1));
    _mm256_cvtss_f32(v)
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

#[cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), target_feature(enable = "avx2"))]
pub unsafe fn dot3_avx2(
    x0: __m256,
    x1: __m256,
    y0: __m256,
    y1: __m256,
    z0: __m256,
    z1: __m256,
) -> __m256 {
    let mut dot = _mm256_mul_ps(x0, x1);
    dot = _mm256_add_ps(dot, _mm256_mul_ps(y0, y1));
    dot = _mm256_add_ps(dot, _mm256_mul_ps(z0, z1));
    dot
}
