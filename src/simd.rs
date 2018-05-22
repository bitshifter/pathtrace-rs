#![allow(dead_code)]

// re-export fallback scalar code
#[cfg(not(any(target_feature = "sse2", target_feature = "avx2")))]
pub use self::m32::*;

// re-export sse2 if no avx2
#[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
pub use self::m128::*;

// re-export avx2
#[cfg(target_feature = "avx2")]
pub use self::m256::*;

impl ArrayF32xN {
    #[inline]
    pub fn new(v: [f32; VECTOR_WIDTH_DWORDS]) -> ArrayF32xN {
        ArrayF32xN(v)
    }

    #[inline]
    pub fn load(&self) -> f32xN {
        f32xN::load_aligned(&self)
    }

    #[inline]
    pub fn store(&mut self, v: f32xN) {
        v.store_aligned(self)
    }
}

impl ArrayI32xN {
    #[inline]
    pub fn new(v: [i32; VECTOR_WIDTH_DWORDS]) -> ArrayI32xN {
        ArrayI32xN(v)
    }

    #[inline]
    pub fn load(&self) -> i32xN {
        i32xN::load_aligned(&self)
    }

    #[inline]
    pub fn store(&mut self, v: i32xN) {
        v.store_aligned(self)
    }
}

// 128 bit wide simd
#[cfg(target_feature = "sse2")]
mod m128 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    use std::convert::From;
    use std::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};

    #[macro_export]
    macro_rules! _mm_shuffle {
        ($z:expr, $y:expr, $x:expr, $w:expr) => {
            ($z << 6) | ($y << 4) | ($x << 2) | $w
        };
    }

    pub const VECTOR_WIDTH_BITS: usize = 128;
    pub const VECTOR_WIDTH_DWORDS: usize = 4;
    pub const VECTOR_WIDTH_DWORDS_LOG2: usize = 2;

    #[repr(C, align(16))]
    #[derive(Copy, Clone, Debug)]
    pub struct ArrayF32xN(pub [f32; 4]);

    #[repr(C, align(16))]
    #[derive(Copy, Clone, Debug)]
    pub struct ArrayI32xN(pub [i32; 4]);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct f32xN(pub __m128);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct i32xN(pub __m128i);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct b32xN(pub __m128);

    pub fn print_version() {
        println!("Using SSE2 SIMD");
    }

    impl i32xN {
        #[inline]
        pub fn new(e3: i32, e2: i32, e1: i32, e0: i32) -> Self {
            unsafe { i32xN(_mm_set_epi32(e3, e2, e1, e0)) }
        }

        #[inline]
        pub fn splat(i: i32) -> Self {
            unsafe { i32xN(_mm_set1_epi32(i)) }
        }

        #[inline]
        pub fn load_aligned(a: &ArrayI32xN) -> Self {
            unsafe { i32xN(_mm_load_si128(a.0.as_ptr() as *const __m128i)) }
        }

        #[inline]
        pub fn load_unaligned(a: &[i32; 4]) -> Self {
            unsafe { i32xN(_mm_loadu_si128(a.as_ptr() as *const __m128i)) }
        }

        #[inline]
        pub fn store_aligned(self, a: &mut ArrayI32xN) {
            unsafe { _mm_store_si128(a.0.as_mut_ptr() as *mut __m128i, self.0) }
        }

        #[inline]
        pub fn store_unaligned(self, a: &mut [i32; 4]) {
            unsafe { _mm_storeu_si128(a.as_mut_ptr() as *mut __m128i, self.0) }
        }

        #[inline]
        // returns an i32xN with each lane set to it's index number
        // TODO: maybe there is a better way to do this...
        pub fn indices() -> Self {
            Self::new(3, 2, 1, 0)
        }

        #[inline]
        // TODO: might feel better as a free function
        // TODO: sse2 implementation
        pub fn blend(self: Self, rhs: Self, cond: b32xN) -> Self {
            #[cfg(target_feature = "sse4.1")]
            {
                unsafe { i32xN(_mm_blendv_epi8(self.0, rhs.0, _mm_castps_si128(cond.0))) }
            }
            #[cfg(not(target_feature = "sse4.1"))]
            {
                unsafe {
                    let d = _mm_srai_epi32(_mm_castps_si128(cond.0), 31);
                    i32xN(_mm_or_si128(
                        _mm_and_si128(d, rhs.0),
                        _mm_andnot_si128(d, self.0),
                    ))
                }
            }
        }
    }

    impl From<i32> for i32xN {
        #[inline]
        fn from(i: i32) -> i32xN {
            i32xN::splat(i)
        }
    }

    impl Add for i32xN {
        type Output = i32xN;
        #[inline]
        fn add(self, rhs: i32xN) -> i32xN {
            unsafe { i32xN(_mm_add_epi32(self.0, rhs.0)) }
        }
    }

    impl Mul<i32xN> for i32xN {
        type Output = i32xN;
        #[inline]
        fn mul(self, rhs: i32xN) -> i32xN {
            unsafe { i32xN(_mm_mul_epi32(self.0, rhs.0)) }
        }
    }

    impl Sub for i32xN {
        type Output = i32xN;
        #[inline]
        fn sub(self, rhs: i32xN) -> i32xN {
            unsafe { i32xN(_mm_sub_epi32(self.0, rhs.0)) }
        }
    }

    impl b32xN {
        pub fn to_mask(self) -> i32 {
            unsafe { _mm_movemask_ps(self.0) }
        }
    }

    impl BitAnd for b32xN {
        type Output = Self;
        fn bitand(self, rhs: Self) -> Self::Output {
            unsafe { b32xN(_mm_and_ps(self.0, rhs.0)) }
        }
    }

    impl BitOr for b32xN {
        type Output = Self;
        fn bitor(self, rhs: Self) -> Self::Output {
            unsafe { b32xN(_mm_or_ps(self.0, rhs.0)) }
        }
    }

    impl From<b32xN> for i32 {
        fn from(b: b32xN) -> i32 {
            unsafe { _mm_movemask_ps(b.0) }
        }
    }

    impl f32xN {
        #[inline]
        pub fn new(e3: f32, e2: f32, e1: f32, e0: f32) -> Self {
            unsafe { f32xN(_mm_set_ps(e3, e2, e1, e0)) }
        }

        #[inline]
        pub fn splat(s: f32) -> Self {
            unsafe { f32xN(_mm_set_ps1(s)) }
        }

        #[inline]
        pub fn load_aligned(a: &ArrayF32xN) -> Self {
            unsafe { f32xN(_mm_load_ps(a.0.as_ptr())) }
        }

        #[inline]
        pub fn load_unaligned(a: &[f32; 4]) -> Self {
            unsafe { f32xN(_mm_loadu_ps(a.as_ptr())) }
        }

        #[inline]
        pub fn store_aligned(self, a: &mut ArrayF32xN) {
            unsafe { _mm_store_ps(a.0.as_mut_ptr(), self.0) }
        }

        #[inline]
        pub fn store_unaligned(self, a: &mut [f32; 4]) {
            unsafe { _mm_storeu_ps(a.as_mut_ptr(), self.0) }
        }

        #[inline]
        pub fn sqrt(self) -> Self {
            unsafe { f32xN(_mm_sqrt_ps(self.0)) }
        }

        #[inline]
        pub fn hmin(self) -> f32 {
            let mut v = self.0;
            unsafe {
                v = _mm_min_ps(v, _mm_shuffle_ps(v, v, _mm_shuffle!(0, 0, 3, 2)));
                v = _mm_min_ps(v, _mm_shuffle_ps(v, v, _mm_shuffle!(0, 0, 0, 1)));
                _mm_cvtss_f32(v)
            }
        }

        #[inline]
        pub fn eq(self, rhs: Self) -> b32xN {
            unsafe { b32xN(_mm_cmpeq_ps(self.0, rhs.0)) }
        }
        #[inline]
        pub fn gt(self, rhs: Self) -> b32xN {
            unsafe { b32xN(_mm_cmpgt_ps(self.0, rhs.0)) }
        }

        #[inline]
        pub fn lt(self, rhs: Self) -> b32xN {
            unsafe { b32xN(_mm_cmplt_ps(self.0, rhs.0)) }
        }

        #[inline]
        // TODO: might feel better as a free function
        pub fn blend(self: f32xN, rhs: f32xN, cond: b32xN) -> f32xN {
            #[cfg(target_feature = "sse4.1")]
            {
                unsafe { f32xN(_mm_blendv_ps(self.0, rhs.0, cond.0)) }
            }
            #[cfg(not(target_feature = "sse4.1"))]
            {
                unsafe {
                    let d = _mm_castsi128_ps(_mm_srai_epi32(_mm_castps_si128(cond.0), 31));
                    f32xN(_mm_or_ps(_mm_and_ps(d, rhs.0), _mm_andnot_ps(d, self.0)))
                }
            }
        }
    }

    impl From<f32> for f32xN {
        #[inline]
        fn from(f: f32) -> Self {
            f32xN::splat(f)
        }
    }

    impl<'a> From<&'a ArrayF32xN> for f32xN {
        #[inline]
        fn from(a: &'a ArrayF32xN) -> Self {
            f32xN::load_aligned(&a)
        }
    }

    #[inline]
    pub fn dot3(x0: f32xN, x1: f32xN, y0: f32xN, y1: f32xN, z0: f32xN, z1: f32xN) -> f32xN {
        unsafe {
            let mut dot = _mm_mul_ps(x0.0, x1.0);
            dot = _mm_add_ps(dot, _mm_mul_ps(y0.0, y1.0));
            dot = _mm_add_ps(dot, _mm_mul_ps(z0.0, z1.0));
            f32xN(dot)
        }
    }

    impl Add for f32xN {
        type Output = f32xN;
        #[inline]
        fn add(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(_mm_add_ps(self.0, rhs.0)) }
        }
    }

    impl Div<f32xN> for f32xN {
        type Output = f32xN;
        #[inline]
        fn div(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(_mm_div_ps(self.0, rhs.0)) }
        }
    }

    impl Mul<f32xN> for f32xN {
        type Output = f32xN;
        #[inline]
        fn mul(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(_mm_mul_ps(self.0, rhs.0)) }
        }
    }

    impl Sub for f32xN {
        type Output = f32xN;
        #[inline]
        fn sub(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(_mm_sub_ps(self.0, rhs.0)) }
        }
    }
}

// 256 bit wide simd
#[cfg(target_feature = "avx2")]
mod m256 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    use std::convert::From;
    use std::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};

    pub const VECTOR_WIDTH_BITS: usize = 256;
    pub const VECTOR_WIDTH_DWORDS: usize = 8;
    pub const VECTOR_WIDTH_DWORDS_LOG2: usize = 3;

    #[repr(C, align(32))]
    #[derive(Copy, Clone, Debug)]
    pub struct ArrayF32xN(pub [f32; 8]);

    #[repr(C, align(32))]
    #[derive(Copy, Clone, Debug)]
    pub struct ArrayI32xN(pub [i32; 8]);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct f32xN(pub __m256);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct i32xN(pub __m256i);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct b32xN(pub __m256);

    pub fn print_version() {
        println!("Using AVX2 SIMD");
    }

    impl i32xN {
        #[inline]
        pub fn new(e7: i32, e6: i32, e5: i32, e4: i32, e3: i32, e2: i32, e1: i32, e0: i32) -> Self {
            unsafe { i32xN(_mm256_set_epi32(e7, e6, e5, e4, e3, e2, e1, e0)) }
        }

        #[inline]
        pub fn splat(i: i32) -> Self {
            unsafe { i32xN(_mm256_set1_epi32(i)) }
        }

        #[inline]
        pub fn load_aligned(a: &ArrayI32xN) -> Self {
            unsafe { i32xN(_mm256_load_si256(a.0.as_ptr() as *const __m256i)) }
        }

        #[inline]
        pub fn load_unaligned(a: &[i32; 8]) -> Self {
            unsafe { i32xN(_mm256_loadu_si256(a.as_ptr() as *const __m256i)) }
        }

        #[inline]
        pub fn store_aligned(self, a: &mut ArrayI32xN) {
            unsafe { _mm256_store_si256(a.0.as_mut_ptr() as *mut __m256i, self.0) }
        }

        #[inline]
        pub fn store_unaligned(self, a: &mut [i32; 8]) {
            unsafe { _mm256_storeu_si256(a.as_mut_ptr() as *mut __m256i, self.0) }
        }

        #[inline]
        // returns an i32xN with each lane set to it's index number
        // TODO: maybe there is a better way to do this...
        pub fn indices() -> Self {
            Self::new(7, 6, 5, 4, 3, 2, 1, 0)
        }

        #[inline]
        // TODO: might feel better as a free function
        pub fn blend(self: Self, rhs: Self, cond: b32xN) -> Self {
            unsafe {
                i32xN(_mm256_blendv_epi8(
                    self.0,
                    rhs.0,
                    _mm256_castps_si256(cond.0),
                ))
            }
        }
    }

    impl From<i32> for i32xN {
        #[inline]
        fn from(i: i32) -> i32xN {
            i32xN::splat(i)
        }
    }

    impl Add for i32xN {
        type Output = i32xN;
        #[inline]
        fn add(self, rhs: i32xN) -> i32xN {
            unsafe { i32xN(_mm256_add_epi32(self.0, rhs.0)) }
        }
    }

    impl Mul<i32xN> for i32xN {
        type Output = i32xN;
        #[inline]
        fn mul(self, rhs: i32xN) -> i32xN {
            unsafe { i32xN(_mm256_mul_epi32(self.0, rhs.0)) }
        }
    }

    impl Sub for i32xN {
        type Output = i32xN;
        #[inline]
        fn sub(self, rhs: i32xN) -> i32xN {
            unsafe { i32xN(_mm256_sub_epi32(self.0, rhs.0)) }
        }
    }

    impl b32xN {
        pub fn to_mask(self) -> i32 {
            unsafe { _mm256_movemask_ps(self.0) }
        }
    }

    impl BitAnd for b32xN {
        type Output = Self;
        fn bitand(self, rhs: Self) -> Self::Output {
            unsafe { b32xN(_mm256_and_ps(self.0, rhs.0)) }
        }
    }

    impl BitOr for b32xN {
        type Output = Self;
        fn bitor(self, rhs: Self) -> Self::Output {
            unsafe { b32xN(_mm256_or_ps(self.0, rhs.0)) }
        }
    }

    impl From<b32xN> for i32 {
        fn from(b: b32xN) -> i32 {
            unsafe { _mm256_movemask_ps(b.0) }
        }
    }

    impl f32xN {
        #[inline]
        pub fn new(e7: f32, e6: f32, e5: f32, e4: f32, e3: f32, e2: f32, e1: f32, e0: f32) -> Self {
            unsafe { f32xN(_mm256_set_ps(e7, e6, e5, e4, e3, e2, e1, e0)) }
        }

        #[inline]
        pub fn splat(s: f32) -> Self {
            unsafe { f32xN(_mm256_set1_ps(s)) }
        }

        #[inline]
        pub fn load_aligned(a: &ArrayF32xN) -> Self {
            unsafe { f32xN(_mm256_load_ps(a.0.as_ptr())) }
        }

        #[inline]
        pub fn load_unaligned(a: &[f32; 8]) -> Self {
            unsafe { f32xN(_mm256_loadu_ps(a.as_ptr())) }
        }

        #[inline]
        pub fn store_aligned(self, a: &mut ArrayF32xN) {
            unsafe { _mm256_store_ps(a.0.as_mut_ptr(), self.0) }
        }

        #[inline]
        pub fn store_unaligned(self, a: &mut [f32; 8]) {
            unsafe { _mm256_storeu_ps(a.as_mut_ptr(), self.0) }
        }

        #[inline]
        pub fn sqrt(self) -> Self {
            unsafe { f32xN(_mm256_sqrt_ps(self.0)) }
        }

        #[inline]
        pub fn hmin(self) -> f32 {
            // TODO: write an avx2 hmin
            use std::f32;
            let mut min = f32::MAX;
            let mut data = ArrayF32xN::new([f32::MAX; VECTOR_WIDTH_DWORDS]);
            data.store(self);
            for val in data.0.iter() {
                if *val < min {
                    min = *val;
                }
            }
            min
        }

        #[inline]
        pub fn eq(self, rhs: Self) -> b32xN {
            // _CMP_EQ_OQ    0x00 /* Equal (ordered, non-signaling)  */
            const CMP_EQ_OQ: i32 = 0x00;
            unsafe { b32xN(_mm256_cmp_ps(self.0, rhs.0, CMP_EQ_OQ)) }
        }
        #[inline]
        pub fn gt(self, rhs: Self) -> b32xN {
            // _CMP_GT_OQ    0x1e /* Greater-than (ordered, non-signaling)  */
            const CMP_GT_OQ: i32 = 0x1e;
            unsafe { b32xN(_mm256_cmp_ps(self.0, rhs.0, CMP_GT_OQ)) }
        }

        #[inline]
        pub fn lt(self, rhs: Self) -> b32xN {
            // _CMP_LT_OQ    0x11 /* Less-than (ordered, non-signaling)  */
            const CMP_LT_OQ: i32 = 0x11;
            unsafe { b32xN(_mm256_cmp_ps(self.0, rhs.0, CMP_LT_OQ)) }
        }

        #[inline]
        // TODO: might feel better as a free function
        pub fn blend(self: f32xN, rhs: f32xN, cond: b32xN) -> f32xN {
            unsafe { f32xN(_mm256_blendv_ps(self.0, rhs.0, cond.0)) }
        }
    }

    impl From<f32> for f32xN {
        #[inline]
        fn from(f: f32) -> Self {
            f32xN::splat(f)
        }
    }

    impl<'a> From<&'a ArrayF32xN> for f32xN {
        #[inline]
        fn from(a: &'a ArrayF32xN) -> Self {
            f32xN::load_aligned(&a)
        }
    }

    #[inline]
    pub fn dot3(x0: f32xN, x1: f32xN, y0: f32xN, y1: f32xN, z0: f32xN, z1: f32xN) -> f32xN {
        unsafe {
            let mut dot = _mm256_mul_ps(x0.0, x1.0);
            dot = _mm256_add_ps(dot, _mm256_mul_ps(y0.0, y1.0));
            dot = _mm256_add_ps(dot, _mm256_mul_ps(z0.0, z1.0));
            f32xN(dot)
        }
    }

    impl Add for f32xN {
        type Output = f32xN;
        #[inline]
        fn add(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(_mm256_add_ps(self.0, rhs.0)) }
        }
    }

    impl Div<f32xN> for f32xN {
        type Output = f32xN;
        #[inline]
        fn div(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(_mm256_div_ps(self.0, rhs.0)) }
        }
    }

    impl Mul<f32xN> for f32xN {
        type Output = f32xN;
        #[inline]
        fn mul(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(_mm256_mul_ps(self.0, rhs.0)) }
        }
    }

    impl Sub for f32xN {
        type Output = f32xN;
        #[inline]
        fn sub(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(_mm256_sub_ps(self.0, rhs.0)) }
        }
    }
}

mod m32 {
    use std::intrinsics::{fadd_fast, fdiv_fast, fmul_fast, fsub_fast};
    use std::ops::{Add, BitAnd, BitOr, Div, Mul, Sub};

    pub const VECTOR_WIDTH_BITS: usize = 32;
    pub const VECTOR_WIDTH_DWORDS: usize = 1;
    pub const VECTOR_WIDTH_DWORDS_LOG2: usize = 0;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct ArrayF32xN(pub [f32; 1]);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct ArrayI32xN(pub [i32; 1]);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct f32xN(pub f32);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct i32xN(pub i32);

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct b32xN(pub bool);

    pub fn print_version() {
        println!("Using scalar fallback version");
    }

    impl i32xN {
        #[inline]
        pub fn new(e0: i32) -> Self {
            i32xN(e0)
        }

        #[inline]
        pub fn splat(i: i32) -> Self {
            i32xN(i)
        }

        #[inline]
        pub fn load_aligned(a: &ArrayI32xN) -> Self {
            i32xN(a.0[0])
        }

        #[inline]
        pub fn load_unaligned(a: &ArrayI32xN) -> Self {
            i32xN(a.0[0])
        }

        #[inline]
        pub fn store_aligned(self, a: &mut ArrayI32xN) {
            a.0[0] = self.0
        }

        #[inline]
        pub fn store_unaligned(self, a: &mut ArrayI32xN) {
            a.0[0] = self.0
        }

        #[inline]
        // returns an i32xN with each lane set to it's index number
        // TODO: maybe there is a better way to do this...
        pub fn indices() -> Self {
            Self::new(0)
        }

        #[inline]
        // TODO: might feel better as a free function
        pub fn blend(self: Self, rhs: Self, cond: b32xN) -> Self {
            if cond.0 {
                rhs
            } else {
                self
            }
        }
    }

    impl From<i32> for i32xN {
        #[inline]
        fn from(i: i32) -> i32xN {
            i32xN::splat(i)
        }
    }

    impl Add for i32xN {
        type Output = i32xN;
        #[inline]
        fn add(self, rhs: i32xN) -> i32xN {
            i32xN(self.0 + rhs.0)
        }
    }

    impl Mul<i32xN> for i32xN {
        type Output = i32xN;
        #[inline]
        fn mul(self, rhs: i32xN) -> i32xN {
            i32xN(self.0 * rhs.0)
        }
    }

    impl Sub for i32xN {
        type Output = i32xN;
        #[inline]
        fn sub(self, rhs: i32xN) -> i32xN {
            i32xN(self.0 - rhs.0)
        }
    }

    impl b32xN {
        pub fn to_mask(self) -> i32 {
            self.0 as i32
        }
    }

    impl BitAnd for b32xN {
        type Output = Self;
        fn bitand(self, rhs: Self) -> Self::Output {
            b32xN(self.0 && rhs.0)
        }
    }

    impl BitOr for b32xN {
        type Output = Self;
        fn bitor(self, rhs: Self) -> Self::Output {
            b32xN(self.0 || rhs.0)
        }
    }

    impl From<b32xN> for i32 {
        fn from(b: b32xN) -> i32 {
            b.to_mask()
        }
    }

    impl f32xN {
        #[inline]
        pub fn new(e0: f32) -> Self {
            f32xN(e0)
        }

        #[inline]
        pub fn splat(s: f32) -> Self {
            f32xN(s)
        }

        #[inline]
        pub fn load_aligned(a: &ArrayF32xN) -> Self {
            f32xN(a.0[0])
        }

        #[inline]
        pub fn load_unaligned(a: &ArrayF32xN) -> Self {
            f32xN(a.0[0])
        }

        #[inline]
        pub fn store_aligned(self, a: &mut ArrayF32xN) {
            a.0[0] = self.0
        }

        #[inline]
        pub fn store_unaligned(self, a: &mut ArrayF32xN) {
            a.0[0] = self.0
        }

        #[inline]
        pub fn sqrt(self) -> Self {
            f32xN(self.0.sqrt())
        }

        #[inline]
        pub fn hmin(self) -> f32 {
            self.0
        }

        #[inline]
        pub fn eq(self, rhs: Self) -> b32xN {
            b32xN(self.0 == rhs.0)
        }
        #[inline]
        pub fn gt(self, rhs: Self) -> b32xN {
            b32xN(self.0 > rhs.0)
        }

        #[inline]
        pub fn lt(self, rhs: Self) -> b32xN {
            b32xN(self.0 < rhs.0)
        }

        #[inline]
        // TODO: might feel better as a free function
        pub fn blend(self: f32xN, rhs: f32xN, cond: b32xN) -> f32xN {
            if cond.0 {
                f32xN(rhs.0)
            } else {
                f32xN(self.0)
            }
        }
    }

    impl From<f32> for f32xN {
        #[inline]
        fn from(f: f32) -> Self {
            f32xN::splat(f)
        }
    }

    impl<'a> From<&'a ArrayF32xN> for f32xN {
        #[inline]
        fn from(a: &'a ArrayF32xN) -> Self {
            f32xN::load_aligned(&a)
        }
    }

    #[inline]
    pub fn dot3(x0: f32xN, x1: f32xN, y0: f32xN, y1: f32xN, z0: f32xN, z1: f32xN) -> f32xN {
        x0 * x1 + y0 * y1 + z0 * z1
    }

    impl Add for f32xN {
        type Output = f32xN;
        #[inline]
        fn add(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(fadd_fast(self.0, rhs.0)) }
        }
    }

    impl Div<f32xN> for f32xN {
        type Output = f32xN;
        #[inline]
        fn div(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(fdiv_fast(self.0, rhs.0)) }
        }
    }

    impl Mul<f32xN> for f32xN {
        type Output = f32xN;
        #[inline]
        fn mul(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(fmul_fast(self.0, rhs.0)) }
        }
    }

    impl Sub for f32xN {
        type Output = f32xN;
        #[inline]
        fn sub(self, rhs: f32xN) -> f32xN {
            unsafe { f32xN(fsub_fast(self.0, rhs.0)) }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_feature = "sse2")]
    mod m128 {
        use test::{black_box, Bencher};
        use simd::m128::*;
        #[test]
        fn test_hmin() {
            assert_eq!(1.0, f32xN::new(1.0, 2.0, 3.0, 4.0).hmin());
            assert_eq!(1.0, f32xN::new(2.0, 3.0, 4.0, 1.0).hmin());
            assert_eq!(1.0, f32xN::new(3.0, 4.0, 1.0, 2.0).hmin());
            assert_eq!(1.0, f32xN::new(4.0, 1.0, 2.0, 3.0).hmin());
        }

        #[bench]
        fn bench_hmin(b: &mut Bencher) {
            use rand::{weak_rng, Rng};
            let mut rng = black_box(weak_rng());
            b.iter(|| {
                let r = f32xN::load_unaligned(&rng.gen::<[f32;4]>());
                r.hmin()
            });
        }
    }

    #[cfg(target_feature = "avx2")]
    mod m256 {
        use test::{black_box, Bencher};
        use simd::m256::*;
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

        #[bench]
        fn bench_hmin(b: &mut Bencher) {
            use rand::{weak_rng, Rng};
            let mut rng = black_box(weak_rng());
            b.iter(|| {
                let r = f32xN::load_unaligned(&rng.gen::<[f32;8]>());
                r.hmin()
            });
        }
    }
}
