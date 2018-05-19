// TODO: remove
#![allow(dead_code)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
// use std::cmp::Ordering;
// use std::ops::{Index, IndexMut};
// use std::slice;

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

/*
impl From<f32> for AlignedF32  {
    fn from(f: f32) -> AlignedF32 {
        AlignedF32(f)
    }
}

impl From<AlignedF32> for f32 {
    fn from(f: AlignedF32) -> f32 {
        f.0
    }
}

// HACK: work around f32's lack of Ord
impl Ord for AlignedF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 > other.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for AlignedF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AlignedF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
*/

// impl Into<AlignedF32> for f32 {
//     fn into(self) -> AlignedF32 {
//         AlignedF32(self)
//     }
// }

impl ArrayF32xN {
    #[inline]
    pub fn new(v: [f32; F32XN_LANES]) -> ArrayF32xN {
        ArrayF32xN(v)
    }

    #[inline]
    pub fn zero() -> ArrayF32xN {
        ArrayF32xN([0.0; F32XN_LANES])
    }
    
    /*
    #[inline]
    pub fn as_ptr(&self) -> *const f32 {
        self.0.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut f32 {
        self.0.as_mut_ptr()
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<'_, f32> {
        self.0.iter()
    }
    */
}

/*
impl Index<usize> for ArrayF32xN {
    type Output = f32;
    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl IndexMut<usize> for ArrayF32xN {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}
*/
