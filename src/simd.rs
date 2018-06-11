#![allow(dead_code)]

use std::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};
use vmath::Vec3;

#[macro_export]
macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

// re-export fallback scalar code
// #[cfg(not(any(target_feature = "sse2", target_feature = "avx2")))]
// pub use self::m32::*;

// re-export sse2 if no avx2
// #[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
// pub use self::m128::*;

// re-export avx2
#[cfg(target_feature = "avx2")]
pub use self::m256::*;

pub trait Bool32xN<T>: Sized + Copy + Clone + BitAnd + BitOr {
    fn num_lanes() -> usize;
    fn unwrap(self) -> T;
    fn to_mask(self) -> i32;
}

pub trait Int32xN<T, BI, B: Bool32xN<BI>>: Sized + Copy + Clone + Add + Sub + Mul {
    fn num_lanes() -> usize;
    fn unwrap(self) -> T;
    fn splat(i: i32) -> Self;
    unsafe fn load_aligned(a: &[i32]) -> Self;
    unsafe fn load_unaligned(a: &[i32]) -> Self;
    unsafe fn store_aligned(self, a: &mut [i32]);
    unsafe fn store_unaligned(self, a: &mut [i32]);
    fn indices() -> Self;
    fn blend(lhs: Self, rhs: Self, cond: B) -> Self;
}

pub trait Float32xN<T, BI, B: Bool32xN<BI>>: Sized + Copy + Clone + Add + Sub + Mul + Div {
    fn num_lanes() -> usize;
    fn unwrap(self) -> T;
    fn splat(s: f32) -> Self;
    fn from_x(v: Vec3) -> Self;
    fn from_y(v: Vec3) -> Self;
    fn from_z(v: Vec3) -> Self;
    unsafe fn load_aligned(a: &[f32]) -> Self;
    unsafe fn load_unaligned(a: &[f32]) -> Self;
    unsafe fn store_aligned(self, a: &mut [f32]);
    unsafe fn store_unaligned(self, a: &mut [f32]);
    fn sqrt(self) -> Self;
    fn hmin(self) -> f32;
    fn eq(self, rhs: Self) -> B;
    fn gt(self, rhs: Self) -> B;
    fn lt(self, rhs: Self) -> B;
    fn dot3(x0: Self, x1: Self, y0: Self, y1: Self, z0: Self, z1: Self) -> Self;
    fn blend(lhs: Self, rhs: Self, cond: B) -> Self;
}

// 128 bit wide simd
// #[cfg(target_feature = "sse2")]
pub mod m128 {
    use simd::{Bool32xN, Float32xN, Int32xN};
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    use std::convert::From;
    use std::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};
    use vmath::sse2::Vec3;

    pub const VECTOR_WIDTH_BITS: usize = 128;
    pub const VECTOR_WIDTH_DWORDS: usize = 4;
    pub const VECTOR_WIDTH_DWORDS_LOG2: usize = 2;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct f32x4(pub __m128);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct i32x4(pub __m128i);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct b32x4(pub __m128);

    pub fn print_version() {
        println!("Using SSE2 SIMD");
    }

    impl Bool32xN<__m128> for b32x4 {
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }
        fn unwrap(self) -> __m128 {
            self.0
        }
        #[target_feature(enable = "sse2")]
        fn to_mask(self) -> i32 {
            unsafe { _mm_movemask_ps(self.0) }
        }
    }

    impl i32x4 {
        #[inline]
        fn new(e3: i32, e2: i32, e1: i32, e0: i32) -> Self {
            unsafe { i32x4(_mm_set_epi32(e3, e2, e1, e0)) }
        }
    }

    impl Int32xN<__m128i, __m128, b32x4> for i32x4 {
        #[inline]
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }

        #[inline]
        fn unwrap(self) -> __m128i {
            self.0
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn splat(i: i32) -> Self {
            unsafe { i32x4(_mm_set1_epi32(i)) }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        unsafe fn load_aligned(a: &[i32]) -> Self {
            debug_assert!(a.len() == Self::num_lanes());
            i32x4(_mm_load_si128(a.as_ptr() as *const __m128i))
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        unsafe fn load_unaligned(a: &[i32]) -> Self {
            debug_assert!(a.len() == Self::num_lanes());
            i32x4(_mm_loadu_si128(a.as_ptr() as *const __m128i))
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        unsafe fn store_aligned(self, a: &mut [i32]) {
            debug_assert!(a.len() == Self::num_lanes());
            _mm_store_si128(a.as_mut_ptr() as *mut __m128i, self.0)
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        unsafe fn store_unaligned(self, a: &mut [i32]) {
            _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0)
        }

        #[inline]
        // returns an i32xN with each lane set to it's index number
        // TODO: maybe there is a better way to do this...
        fn indices() -> Self {
            Self::new(3, 2, 1, 0)
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn blend(lhs: Self, rhs: Self, cond: b32x4) -> Self {
            #[cfg(target_feature = "sse4.1")]
            {
                unsafe { i32x4(_mm_blendv_epi8(lhs.0, rhs.0, _mm_castps_si128(cond.0))) }
            }
            #[cfg(not(target_feature = "sse4.1"))]
            {
                unsafe {
                    let d = _mm_srai_epi32(_mm_castps_si128(cond.unwrap()), 31);
                    i32x4(_mm_or_si128(
                        _mm_and_si128(d, rhs.0),
                        _mm_andnot_si128(d, lhs.0),
                    ))
                }
            }
        }
    }

    impl From<i32> for i32x4 {
        #[inline]
        fn from(i: i32) -> i32x4 {
            i32x4::splat(i)
        }
    }

    impl Add for i32x4 {
        type Output = i32x4;
        #[inline]
        #[target_feature(enable = "sse2")]
        fn add(self, rhs: i32x4) -> i32x4 {
            unsafe { i32x4(_mm_add_epi32(self.0, rhs.0)) }
        }
    }

    impl Mul<i32x4> for i32x4 {
        type Output = i32x4;
        #[inline]
        #[target_feature(enable = "sse2")]
        fn mul(self, rhs: i32x4) -> i32x4 {
            unsafe { i32x4(_mm_mul_epi32(self.0, rhs.0)) }
        }
    }

    impl Sub for i32x4 {
        type Output = i32x4;
        #[inline]
        #[target_feature(enable = "sse2")]
        fn sub(self, rhs: i32x4) -> i32x4 {
            unsafe { i32x4(_mm_sub_epi32(self.0, rhs.0)) }
        }
    }

    impl BitAnd for b32x4 {
        type Output = Self;
        #[target_feature(enable = "sse2")]
        fn bitand(self, rhs: Self) -> Self::Output {
            unsafe { b32x4(_mm_and_ps(self.0, rhs.0)) }
        }
    }

    impl BitOr for b32x4 {
        type Output = Self;
        #[target_feature(enable = "sse2")]
        fn bitor(self, rhs: Self) -> Self::Output {
            unsafe { b32x4(_mm_or_ps(self.0, rhs.0)) }
        }
    }

    impl From<b32x4> for i32 {
        #[target_feature(enable = "sse2")]
        fn from(b: b32x4) -> i32 {
            unsafe { _mm_movemask_ps(b.0) }
        }
    }

    impl f32x4 {
        #[inline]
        #[target_feature(enable = "sse2")]
        fn new(e3: f32, e2: f32, e1: f32, e0: f32) -> Self {
            unsafe { f32x4(_mm_set_ps(e3, e2, e1, e0)) }
        }
    }

    impl Float32xN<__m128, __m128, b32x4> for f32x4 {
        #[inline]
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }

        #[inline]
        fn splat(s: f32) -> Self {
            unsafe { f32x4(_mm_set_ps1(s)) }
        }

        #[inline]
        fn unwrap(self) -> __m128 {
            self.0
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn from_x(v: Vec3) -> Self {
            let m: Self = v.into();
            unsafe { f32x4(_mm_shuffle_ps(m.0, m.0, _mm_shuffle!(0, 0, 0, 0))) }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn from_y(v: Vec3) -> Self {
            let m: Self = v.into();
            unsafe { f32x4(_mm_shuffle_ps(m.0, m.0, _mm_shuffle!(1, 1, 1, 1))) }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn from_z(v: Vec3) -> Self {
            let m: Self = v.into();
            unsafe { f32x4(_mm_shuffle_ps(m.0, m.0, _mm_shuffle!(2, 2, 2, 2))) }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        unsafe fn load_aligned(a: &[f32]) -> Self {
            f32x4(_mm_load_ps(a.as_ptr()))
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        unsafe fn load_unaligned(a: &[f32]) -> Self {
            f32x4(_mm_loadu_ps(a.as_ptr()))
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        unsafe fn store_aligned(self, a: &mut [f32]) {
            _mm_store_ps(a.as_mut_ptr(), self.0)
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        unsafe fn store_unaligned(self, a: &mut [f32]) {
            _mm_storeu_ps(a.as_mut_ptr(), self.0)
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn sqrt(self) -> Self {
            unsafe { f32x4(_mm_sqrt_ps(self.0)) }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn hmin(self) -> f32 {
            let mut v = self.0;
            unsafe {
                v = _mm_min_ps(v, _mm_shuffle_ps(v, v, _mm_shuffle!(0, 0, 3, 2)));
                v = _mm_min_ps(v, _mm_shuffle_ps(v, v, _mm_shuffle!(0, 0, 0, 1)));
                _mm_cvtss_f32(v)
            }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn eq(self, rhs: Self) -> b32x4 {
            unsafe { b32x4(_mm_cmpeq_ps(self.0, rhs.0)) }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn gt(self, rhs: Self) -> b32x4 {
            unsafe { b32x4(_mm_cmpgt_ps(self.0, rhs.0)) }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn lt(self, rhs: Self) -> b32x4 {
            unsafe { b32x4(_mm_cmplt_ps(self.0, rhs.0)) }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn blend(lhs: f32x4, rhs: f32x4, cond: b32x4) -> f32x4 {
            #[cfg(target_feature = "sse4.1")]
            {
                unsafe { f32x4(_mm_blendv_ps(lhs.0, rhs.0, cond.0)) }
            }
            #[cfg(not(target_feature = "sse4.1"))]
            {
                unsafe {
                    let d = _mm_castsi128_ps(_mm_srai_epi32(_mm_castps_si128(cond.0), 31));
                    f32x4(_mm_or_ps(_mm_and_ps(d, rhs.0), _mm_andnot_ps(d, lhs.0)))
                }
            }
        }

        #[inline]
        #[target_feature(enable = "sse2")]
        fn dot3(x0: f32x4, x1: f32x4, y0: f32x4, y1: f32x4, z0: f32x4, z1: f32x4) -> f32x4 {
            unsafe {
                let mut dot = _mm_mul_ps(x0.0, x1.0);
                dot = _mm_add_ps(dot, _mm_mul_ps(y0.0, y1.0));
                dot = _mm_add_ps(dot, _mm_mul_ps(z0.0, z1.0));
                f32x4(dot)
            }
        }
    }

    impl From<f32> for f32x4 {
        #[inline]
        fn from(f: f32) -> Self {
            f32x4::splat(f)
        }
    }

    impl Add for f32x4 {
        type Output = f32x4;
        #[inline]
        #[target_feature(enable = "sse2")]
        fn add(self, rhs: f32x4) -> f32x4 {
            unsafe { f32x4(_mm_add_ps(self.0, rhs.0)) }
        }
    }

    impl Div<f32x4> for f32x4 {
        type Output = f32x4;
        #[inline]
        #[target_feature(enable = "sse2")]
        fn div(self, rhs: f32x4) -> f32x4 {
            unsafe { f32x4(_mm_div_ps(self.0, rhs.0)) }
        }
    }

    impl Mul<f32x4> for f32x4 {
        type Output = f32x4;
        #[inline]
        #[target_feature(enable = "sse2")]
        fn mul(self, rhs: f32x4) -> f32x4 {
            unsafe { f32x4(_mm_mul_ps(self.0, rhs.0)) }
        }
    }

    impl Sub for f32x4 {
        type Output = f32x4;
        #[inline]
        #[target_feature(enable = "sse2")]
        fn sub(self, rhs: f32x4) -> f32x4 {
            unsafe { f32x4(_mm_sub_ps(self.0, rhs.0)) }
        }
    }
}

// 256 bit wide simd
// #[cfg(target_feature = "avx2")]
pub mod m256 {
    use simd::m128;
    use simd::{Bool32xN, Float32xN, Int32xN};
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    use std::convert::From;
    use std::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};
    use vmath::Vec3;

    pub const VECTOR_WIDTH_BITS: usize = 256;
    pub const VECTOR_WIDTH_DWORDS: usize = 8;
    pub const VECTOR_WIDTH_DWORDS_LOG2: usize = 3;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct f32x8(pub __m256);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct i32x8(pub __m256i);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct b32x8(pub __m256);

    pub fn print_version() {
        println!("Using AVX2 SIMD");
    }

    impl Bool32xN<__m256> for b32x8 {
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }
        fn unwrap(self) -> __m256 {
            self.0
        }
        #[target_feature(enable = "avx2")]
        fn to_mask(self) -> i32 {
            unsafe { _mm256_movemask_ps(self.0) }
        }
    }

    impl i32x8 {
        #[inline]
        #[target_feature(enable = "avx2")]
        pub fn new(e7: i32, e6: i32, e5: i32, e4: i32, e3: i32, e2: i32, e1: i32, e0: i32) -> Self {
            unsafe { i32x8(_mm256_set_epi32(e7, e6, e5, e4, e3, e2, e1, e0)) }
        }
    }

    impl Int32xN<__m256i, __m256, b32x8> for i32x8 {
        #[inline]
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }

        #[inline]
        fn unwrap(self) -> __m256i {
            self.0
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn splat(i: i32) -> Self {
            unsafe { i32x8(_mm256_set1_epi32(i)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn load_aligned(a: &[i32]) -> Self {
            i32x8(_mm256_load_si256(a.as_ptr() as *const __m256i))
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn load_unaligned(a: &[i32]) -> Self {
            i32x8(_mm256_loadu_si256(a.as_ptr() as *const __m256i))
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn store_aligned(self, a: &mut [i32]) {
            _mm256_store_si256(a.as_mut_ptr() as *mut __m256i, self.0)
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn store_unaligned(self, a: &mut [i32]) {
            _mm256_storeu_si256(a.as_mut_ptr() as *mut __m256i, self.0)
        }

        #[inline]
        // returns an i32x8 with each lane set to it's index number
        // TODO: maybe there is a better way to do this...
        fn indices() -> Self {
            Self::new(7, 6, 5, 4, 3, 2, 1, 0)
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn blend(lhs: Self, rhs: Self, cond: b32x8) -> Self {
            unsafe {
                i32x8(_mm256_blendv_epi8(
                    lhs.0,
                    rhs.0,
                    _mm256_castps_si256(cond.0),
                ))
            }
        }
    }

    impl From<i32> for i32x8 {
        #[inline]
        fn from(i: i32) -> i32x8 {
            i32x8::splat(i)
        }
    }

    impl Add for i32x8 {
        type Output = i32x8;
        #[inline]
        #[target_feature(enable = "avx2")]
        fn add(self, rhs: i32x8) -> i32x8 {
            unsafe { i32x8(_mm256_add_epi32(self.0, rhs.0)) }
        }
    }

    impl Mul<i32x8> for i32x8 {
        type Output = i32x8;
        #[inline]
        #[target_feature(enable = "avx2")]
        fn mul(self, rhs: i32x8) -> i32x8 {
            unsafe { i32x8(_mm256_mul_epi32(self.0, rhs.0)) }
        }
    }

    impl Sub for i32x8 {
        type Output = i32x8;
        #[inline]
        #[target_feature(enable = "avx2")]
        fn sub(self, rhs: i32x8) -> i32x8 {
            unsafe { i32x8(_mm256_sub_epi32(self.0, rhs.0)) }
        }
    }

    impl BitAnd for b32x8 {
        type Output = Self;
        #[target_feature(enable = "avx2")]
        fn bitand(self, rhs: Self) -> Self::Output {
            unsafe { b32x8(_mm256_and_ps(self.0, rhs.0)) }
        }
    }

    impl BitOr for b32x8 {
        type Output = Self;
        #[target_feature(enable = "avx2")]
        fn bitor(self, rhs: Self) -> Self::Output {
            unsafe { b32x8(_mm256_or_ps(self.0, rhs.0)) }
        }
    }

    impl From<b32x8> for i32 {
        #[target_feature(enable = "avx2")]
        fn from(b: b32x8) -> i32 {
            unsafe { _mm256_movemask_ps(b.0) }
        }
    }

    impl f32x8 {
        #[inline]
        #[target_feature(enable = "avx2")]
        pub fn new(e7: f32, e6: f32, e5: f32, e4: f32, e3: f32, e2: f32, e1: f32, e0: f32) -> Self {
            unsafe { f32x8(_mm256_set_ps(e7, e6, e5, e4, e3, e2, e1, e0)) }
        }
    }

    impl Float32xN<__m256, __m256, b32x8> for f32x8 {
        #[inline]
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }

        #[inline]
        fn unwrap(self) -> __m256 {
            self.0
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn splat(s: f32) -> Self {
            unsafe { f32x8(_mm256_set1_ps(s)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn from_x(v: Vec3) -> Self {
            let m = m128::f32x4::from_x(v);
            unsafe { f32x8(_mm256_set_m128(m.0, m.0)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn from_y(v: Vec3) -> Self {
            let m = m128::f32x4::from_y(v);
            unsafe { f32x8(_mm256_set_m128(m.0, m.0)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn from_z(v: Vec3) -> Self {
            let m = m128::f32x4::from_z(v);
            unsafe { f32x8(_mm256_set_m128(m.0, m.0)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn load_aligned(a: &[f32]) -> Self {
            f32x8(_mm256_load_ps(a.as_ptr()))
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn load_unaligned(a: &[f32]) -> Self {
            f32x8(_mm256_loadu_ps(a.as_ptr()))
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn store_aligned(self, a: &mut [f32]) {
            _mm256_store_ps(a.as_mut_ptr(), self.0)
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        unsafe fn store_unaligned(self, a: &mut [f32]) {
            _mm256_storeu_ps(a.as_mut_ptr(), self.0)
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn sqrt(self) -> Self {
            unsafe { f32x8(_mm256_sqrt_ps(self.0)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn hmin(self) -> f32 {
            unsafe {
                let mut v = self.0;
                v = _mm256_min_ps(v, _mm256_permute_ps(v, _mm_shuffle!(0, 0, 3, 2)));
                v = _mm256_min_ps(v, _mm256_permute_ps(v, _mm_shuffle!(0, 0, 0, 1)));
                v = _mm256_min_ps(v, _mm256_permute2f128_ps(v, v, 1));
                _mm256_cvtss_f32(v)
            }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn eq(self, rhs: Self) -> b32x8 {
            // _CMP_EQ_OQ    0x00 /* Equal (ordered, non-signaling)  */
            const CMP_EQ_OQ: i32 = 0x00;
            unsafe { b32x8(_mm256_cmp_ps(self.0, rhs.0, CMP_EQ_OQ)) }
        }
        #[inline]
        #[target_feature(enable = "avx2")]
        fn gt(self, rhs: Self) -> b32x8 {
            // _CMP_GT_OQ    0x1e /* Greater-than (ordered, non-signaling)  */
            const CMP_GT_OQ: i32 = 0x1e;
            unsafe { b32x8(_mm256_cmp_ps(self.0, rhs.0, CMP_GT_OQ)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn lt(self, rhs: Self) -> b32x8 {
            // _CMP_LT_OQ    0x11 /* Less-than (ordered, non-signaling)  */
            const CMP_LT_OQ: i32 = 0x11;
            unsafe { b32x8(_mm256_cmp_ps(self.0, rhs.0, CMP_LT_OQ)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn blend(lhs: f32x8, rhs: f32x8, cond: b32x8) -> f32x8 {
            unsafe { f32x8(_mm256_blendv_ps(lhs.0, rhs.0, cond.0)) }
        }

        #[inline]
        #[target_feature(enable = "avx2")]
        fn dot3(x0: f32x8, x1: f32x8, y0: f32x8, y1: f32x8, z0: f32x8, z1: f32x8) -> f32x8 {
            unsafe {
                let mut dot = _mm256_mul_ps(x0.0, x1.0);
                dot = _mm256_add_ps(dot, _mm256_mul_ps(y0.0, y1.0));
                dot = _mm256_add_ps(dot, _mm256_mul_ps(z0.0, z1.0));
                f32x8(dot)
            }
        }
    }

    impl From<f32> for f32x8 {
        #[inline]
        fn from(f: f32) -> Self {
            f32x8::splat(f)
        }
    }

    impl Add for f32x8 {
        type Output = f32x8;
        #[inline]
        #[target_feature(enable = "avx2")]
        fn add(self, rhs: f32x8) -> f32x8 {
            unsafe { f32x8(_mm256_add_ps(self.0, rhs.0)) }
        }
    }

    impl Div<f32x8> for f32x8 {
        type Output = f32x8;
        #[inline]
        #[target_feature(enable = "avx2")]
        fn div(self, rhs: f32x8) -> f32x8 {
            unsafe { f32x8(_mm256_div_ps(self.0, rhs.0)) }
        }
    }

    impl Mul<f32x8> for f32x8 {
        type Output = f32x8;
        #[inline]
        #[target_feature(enable = "avx2")]
        fn mul(self, rhs: f32x8) -> f32x8 {
            unsafe { f32x8(_mm256_mul_ps(self.0, rhs.0)) }
        }
    }

    impl Sub for f32x8 {
        type Output = f32x8;
        #[inline]
        #[target_feature(enable = "avx2")]
        fn sub(self, rhs: f32x8) -> f32x8 {
            unsafe { f32x8(_mm256_sub_ps(self.0, rhs.0)) }
        }
    }
}

pub mod m32 {
    use simd::{Bool32xN, Float32xN, Int32xN};
    use std::intrinsics::{fadd_fast, fdiv_fast, fmul_fast, fsub_fast};
    use std::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};
    use vmath::Vec3;

    pub const VECTOR_WIDTH_BITS: usize = 32;
    pub const VECTOR_WIDTH_DWORDS: usize = 1;
    pub const VECTOR_WIDTH_DWORDS_LOG2: usize = 0;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct f32x1(pub f32);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct i32x1(pub i32);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct b32x1(pub bool);

    pub fn print_version() {
        println!("Using scalar fallback version");
    }

    impl Bool32xN<bool> for b32x1 {
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }
        fn unwrap(self) -> bool {
            self.0
        }
        fn to_mask(self) -> i32 {
            self.0 as i32
        }
    }

    impl i32x1 {
        #[inline]
        pub fn new(e0: i32) -> Self {
            i32x1(e0)
        }
    }

    impl Int32xN<i32, bool, b32x1> for i32x1 {
        #[inline]
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }

        #[inline]
        fn unwrap(self) -> i32 {
            self.0
        }

        #[inline]
        fn splat(i: i32) -> Self {
            i32x1(i)
        }

        #[inline]
        unsafe fn load_aligned(a: &[i32]) -> Self {
            i32x1(a[0])
        }

        #[inline]
        unsafe fn load_unaligned(a: &[i32]) -> Self {
            i32x1(a[0])
        }

        #[inline]
        unsafe fn store_aligned(self, a: &mut [i32]) {
            a[0] = self.0
        }

        #[inline]
        unsafe fn store_unaligned(self, a: &mut [i32]) {
            a[0] = self.0
        }

        #[inline]
        // returns an i32x1 with each lane set to it's index number
        // TODO: maybe there is a better way to do this...
        fn indices() -> Self {
            Self::new(0)
        }

        #[inline]
        fn blend(lhs: Self, rhs: Self, cond: b32x1) -> Self {
            if cond.0 {
                rhs
            } else {
                lhs
            }
        }
    }

    impl From<i32> for i32x1 {
        #[inline]
        fn from(i: i32) -> i32x1 {
            i32x1::splat(i)
        }
    }

    impl Add for i32x1 {
        type Output = i32x1;
        #[inline]
        fn add(self, rhs: i32x1) -> i32x1 {
            i32x1(self.0 + rhs.0)
        }
    }

    impl Mul<i32x1> for i32x1 {
        type Output = i32x1;
        #[inline]
        fn mul(self, rhs: i32x1) -> i32x1 {
            i32x1(self.0 * rhs.0)
        }
    }

    impl Sub for i32x1 {
        type Output = i32x1;
        #[inline]
        fn sub(self, rhs: i32x1) -> i32x1 {
            i32x1(self.0 - rhs.0)
        }
    }

    impl b32x1 {
        pub fn to_mask(self) -> i32 {
            self.0 as i32
        }
    }

    impl BitAnd for b32x1 {
        type Output = Self;
        fn bitand(self, rhs: Self) -> Self::Output {
            b32x1(self.0 && rhs.0)
        }
    }

    impl BitOr for b32x1 {
        type Output = Self;
        fn bitor(self, rhs: Self) -> Self::Output {
            b32x1(self.0 || rhs.0)
        }
    }

    impl From<b32x1> for i32 {
        fn from(b: b32x1) -> i32 {
            b.to_mask()
        }
    }

    impl f32x1 {
        #[inline]
        pub fn new(e0: f32) -> Self {
            f32x1(e0)
        }
    }

    impl Float32xN<f32, bool, b32x1> for f32x1 {
        #[inline]
        fn num_lanes() -> usize {
            VECTOR_WIDTH_DWORDS
        }

        #[inline]
        fn unwrap(self) -> f32 {
            self.0
        }

        #[inline]
        fn splat(s: f32) -> Self {
            f32x1(s)
        }

        #[inline]
        fn from_x(v: Vec3) -> Self {
            f32x1::splat(v.get_x())
        }

        #[inline]
        fn from_y(v: Vec3) -> Self {
            f32x1::splat(v.get_y())
        }

        #[inline]
        fn from_z(v: Vec3) -> Self {
            f32x1::splat(v.get_z())
        }

        #[inline]
        unsafe fn load_aligned(a: &[f32]) -> Self {
            f32x1(a[0])
        }

        #[inline]
        unsafe fn load_unaligned(a: &[f32]) -> Self {
            f32x1(a[0])
        }

        #[inline]
        unsafe fn store_aligned(self, a: &mut [f32]) {
            a[0] = self.0
        }

        #[inline]
        unsafe fn store_unaligned(self, a: &mut [f32]) {
            a[0] = self.0
        }

        #[inline]
        fn sqrt(self) -> Self {
            f32x1(self.0.sqrt())
        }

        #[inline]
        fn hmin(self) -> f32 {
            self.0
        }

        #[inline]
        fn eq(self, rhs: Self) -> b32x1 {
            b32x1(self.0 == rhs.0)
        }
        #[inline]
        fn gt(self, rhs: Self) -> b32x1 {
            b32x1(self.0 > rhs.0)
        }

        #[inline]
        fn lt(self, rhs: Self) -> b32x1 {
            b32x1(self.0 < rhs.0)
        }

        #[inline]
        fn blend(lhs: f32x1, rhs: f32x1, cond: b32x1) -> f32x1 {
            if cond.0 {
                f32x1(rhs.0)
            } else {
                f32x1(lhs.0)
            }
        }

        #[inline]
        fn dot3(x0: f32x1, x1: f32x1, y0: f32x1, y1: f32x1, z0: f32x1, z1: f32x1) -> f32x1 {
            x0 * x1 + y0 * y1 + z0 * z1
        }
    }

    impl From<f32> for f32x1 {
        #[inline]
        fn from(f: f32) -> Self {
            f32x1::splat(f)
        }
    }

    impl Add for f32x1 {
        type Output = f32x1;
        #[inline]
        fn add(self, rhs: f32x1) -> f32x1 {
            unsafe { f32x1(fadd_fast(self.0, rhs.0)) }
        }
    }

    impl Div<f32x1> for f32x1 {
        type Output = f32x1;
        #[inline]
        fn div(self, rhs: f32x1) -> f32x1 {
            unsafe { f32x1(fdiv_fast(self.0, rhs.0)) }
        }
    }

    impl Mul<f32x1> for f32x1 {
        type Output = f32x1;
        #[inline]
        fn mul(self, rhs: f32x1) -> f32x1 {
            unsafe { f32x1(fmul_fast(self.0, rhs.0)) }
        }
    }

    impl Sub for f32x1 {
        type Output = f32x1;
        #[inline]
        fn sub(self, rhs: f32x1) -> f32x1 {
            unsafe { f32x1(fsub_fast(self.0, rhs.0)) }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_feature = "sse2")]
    mod m128 {
        use simd::m128::*;
        use test::{black_box, Bencher};
        #[test]
        fn test_hmin() {
            assert_eq!(1.0, f32xN::new(1.0, 2.0, 3.0, 4.0).hmin());
            assert_eq!(1.0, f32xN::new(2.0, 3.0, 4.0, 1.0).hmin());
            assert_eq!(1.0, f32xN::new(3.0, 4.0, 1.0, 2.0).hmin());
            assert_eq!(1.0, f32xN::new(4.0, 1.0, 2.0, 3.0).hmin());
        }
    }

    #[cfg(target_feature = "avx2")]
    mod m256 {
        use simd::m256::*;
        use test::{black_box, Bencher};
        #[test]
        fn test_hmin() {
            assert_eq!(
                1.0,
                f32xN::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0).hmin()
            );
            assert_eq!(
                1.0,
                f32xN::new(2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 1.0).hmin()
            );
            assert_eq!(
                1.0,
                f32xN::new(3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 1.0, 2.0).hmin()
            );
            assert_eq!(
                1.0,
                f32xN::new(4.0, 5.0, 6.0, 7.0, 8.0, 1.0, 2.0, 3.0).hmin()
            );
            assert_eq!(
                1.0,
                f32xN::new(5.0, 6.0, 7.0, 8.0, 1.0, 2.0, 3.0, 4.0).hmin()
            );
            assert_eq!(
                1.0,
                f32xN::new(6.0, 7.0, 8.0, 1.0, 2.0, 3.0, 4.0, 5.0).hmin()
            );
            assert_eq!(
                1.0,
                f32xN::new(7.0, 8.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0).hmin()
            );
            assert_eq!(
                1.0,
                f32xN::new(8.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0).hmin()
            );
        }
    }
}
