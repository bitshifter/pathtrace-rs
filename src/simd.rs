// TODO: remove
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

// Idea is to determine width at compile time
// Need to manually enable avx with `cargo rustc -- -C target-feature=+avx`
// Don't know if there's an easier way
// #[cfg(target_feature = "avx")]
// pub const PACKED_WIDTH: usize = 256;

#[cfg(all(target_feature = "sse2", not(target_feature = "avx")))]
pub const F32XN_BITS: usize = 128;

#[cfg(not(target_feature = "sse2"))]
pub const F32XN_BITS: usize = 32;

pub const F32XN_LANES: usize = F32XN_BITS / 32;

// unaligned F32xN storage
// type UnalignedArrayF32xN = [f32; F32XN_LANES];

#[cfg(target_feature = "avx")]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct F32xN(pub __m256);

#[cfg(target_feature = "avx")]
#[repr(C, align(32))]
#[derive(Copy, Clone, Debug)]
pub struct ArrayF32xN(pub [f32; F32XN_LANES]);

// sse2 128 bit version
#[cfg(all(target_feature = "sse2", not(target_feature = "avx")))]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct F32xN(pub __m128);

#[cfg(all(target_feature = "sse2", not(target_feature = "avx")))]
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct ArrayF32xN(pub [f32; F32XN_LANES]);

// fallback scalar version
#[cfg(not(target_feature = "sse2"))]
#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct F32xN(pub f32);

#[cfg(not(target_feature = "sse2"))]
#[repr(C, align(4))]
#[derive(Copy, Clone, Debug)]
pub struct ArrayF32xN(pub [f32; F32XN_LANES]);

impl ArrayF32xN {
    #[inline]
    pub fn new(v: [f32; F32XN_LANES]) -> ArrayF32xN {
        ArrayF32xN(v)
    }

    #[inline]
    pub fn zero() -> ArrayF32xN {
        ArrayF32xN([0.0; F32XN_LANES])
    }
}

#[cfg(target_feature = "avx")]
fn horizontal_min(v: __m256) -> f32 {
    panic!("Not implemented");
}

#[cfg(all(target_feature = "sse2", not(target_feature = "avx")))]
pub unsafe fn horizontal_min(v: __m128) -> f32 {
    let v = _mm_min_ps(v, _mm_shuffle_ps(v, v, _mm_shuffle!(0, 0, 3, 2)));
    let v = _mm_min_ps(v, _mm_shuffle_ps(v, v, _mm_shuffle!(0, 0, 0, 1)));
    _mm_cvtss_f32(v)
}
