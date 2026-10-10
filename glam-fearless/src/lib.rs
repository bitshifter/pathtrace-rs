//! Wide ("vertical") SIMD maths types for the glam ecosystem, built on
//! [`fearless_simd`].
//!
//! Each type is generic over the fearless_simd token `S: Simd`, so a single type
//! serves every SIMD level: `Vec3xN<S>` is 4 lanes wide when `S = Sse2`/`Neon`, 8
//! when `S = Avx2`, and 16 when `S = Avx512`. The level is chosen at runtime with
//! [`dispatch!`], so one code path covers all of them.
//!
//! Note: fearless_simd's scalar fallback is a 4-lane *emulated* backend
//! (`impl Simd for Fallback { type f32s = f32x4<Fallback>; }`), not a 1-lane one, so
//! `dispatch!` never produces a single-lane path.
//!
//! Vectors carry their token (all fearless_simd tokens are zero-sized), which is
//! what allows ordinary operator syntax with no explicit `simd` argument:
//!
//! ```
//! use glam_fearless::{dispatch, prelude::*, Level, Vec3xN};
//!
//! #[fearless_simd_macros::simd]
//! #[inline(always)]
//! fn sum_lengths_impl<S: Simd>(simd: S, xs: &[f32], ys: &[f32], zs: &[f32]) -> f32 {
//!     let n = S::f32s::LEN;
//!     let mut acc = S::f32s::splat(simd, 0.0);
//!     for ((x, y), z) in xs.chunks_exact(n).zip(ys.chunks_exact(n)).zip(zs.chunks_exact(n)) {
//!         let v = Vec3xN::from_slice(simd, x, y, z);
//!         acc += v.length();
//!     }
//!     acc.reduce_sum()
//! }
//!
//! pub fn sum_lengths(level: Level, xs: &[f32], ys: &[f32], zs: &[f32]) -> f32 {
//!     dispatch!(level, simd => sum_lengths_impl(simd, xs, ys, zs))
//! }
//! ```

pub mod bvec;
pub mod vec;

pub use bvec::{BVec2xN, BVec3xN, BVec4xN};
pub use vec::{Vec2xN, Vec3xN, Vec4xN};

pub use fearless_simd::{self, Level, dispatch, prelude, prelude::*};
pub use fearless_simd_macros::simd;

/// The number of `f32` lanes the given level processes at a time (`S::f32s::LEN`).
#[simd]
#[inline(always)]
pub fn lane_len_f32_impl<S: Simd>(_: S) -> usize {
    S::f32s::LEN
}

/// Number of `f32` lanes for `level`: 4 for the scalar fallback, SSE2, NEON and
/// WASM SIMD; 8 for AVX2; 16 for AVX-512.
///
/// The scalar fallback is a 4-lane emulated backend
/// (`Fallback::f32s = f32x4<Fallback>`), so there is no 1-lane level.
pub fn lane_len_f32(level: Level) -> usize {
    dispatch!(level, simd => lane_len_f32_impl(simd))
}
