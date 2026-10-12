//! Wide quaternion type: one rotation per lane.
//!
//! Rotations that need trigonometry (angles, `slerp`, Euler conversion) are out of scope:
//! `fearless_simd` 1.1 has no vector `sin`/`cos`/`acos`, so a wide quaternion is built from
//! components, most naturally one scalar `glam::Quat` per lane via
//! [`splat_quat`](QuatxN::splat_quat). Multiply, conjugate/inverse, normalize and rotating a
//! [`Vec3xN`] need no trigonometry and are what the type is for.

use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use fearless_simd::prelude::*;
use fearless_simd::{ExtractToken, Simd};

use crate::vec::{Vec3xN, scale3};

/// A wide quaternion; `x`/`y`/`z`/`w` are lane vectors, so one value holds `S::f32s::LEN`
/// independent rotations.
#[derive(Clone, Copy, Debug)]
pub struct QuatxN<S: Simd> {
    /// The X component.
    pub x: S::f32s,
    /// The Y component.
    pub y: S::f32s,
    /// The Z component.
    pub z: S::f32s,
    /// The W (scalar) component.
    pub w: S::f32s,
    token: S,
}

impl<S: Simd> QuatxN<S> {
    /// The SIMD token this value carries.
    #[inline(always)]
    pub fn token(&self) -> S {
        self.token
    }

    /// Construct one lane vector per component.
    #[inline(always)]
    pub fn new(simd: S, x: S::f32s, y: S::f32s, z: S::f32s, w: S::f32s) -> Self {
        Self {
            x,
            y,
            z,
            w,
            token: simd,
        }
    }

    /// Construct from one `f32` per component, splatted across the lanes.
    #[inline(always)]
    pub fn splat_xyzw(simd: S, x: f32, y: f32, z: f32, w: f32) -> Self {
        Self::new(
            simd,
            S::f32s::splat(simd, x),
            S::f32s::splat(simd, y),
            S::f32s::splat(simd, z),
            S::f32s::splat(simd, w),
        )
    }

    /// Construct from a narrow glam quaternion, one rotation broadcast to every lane.
    #[inline(always)]
    pub fn splat_quat(simd: S, value: glam::Quat) -> Self {
        Self::splat_xyzw(simd, value.x, value.y, value.z, value.w)
    }

    /// Construct from one lane slice per component.
    ///
    /// Each slice must be exactly `S::f32s::LEN` long.
    #[inline(always)]
    pub fn from_slice(simd: S, x: &[f32], y: &[f32], z: &[f32], w: &[f32]) -> Self {
        Self::new(
            simd,
            S::f32s::from_slice(simd, x),
            S::f32s::from_slice(simd, y),
            S::f32s::from_slice(simd, z),
            S::f32s::from_slice(simd, w),
        )
    }

    /// The identity rotation in every lane.
    #[inline(always)]
    pub fn identity(simd: S) -> Self {
        Self::splat_xyzw(simd, 0.0, 0.0, 0.0, 1.0)
    }

    /// Dot product of the two quaternions, one result per lane.
    #[inline(always)]
    pub fn dot(self, rhs: Self) -> S::f32s {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }

    /// Squared length of every lane.
    #[inline(always)]
    pub fn length_squared(self) -> S::f32s {
        self.dot(self)
    }

    /// Length of every lane.
    #[inline(always)]
    pub fn length(self) -> S::f32s {
        self.length_squared().sqrt()
    }

    /// Scale every lane to unit length.
    ///
    /// Divides by the length across lanes, as [`Vec3xN::normalize`] does. A lane of zero
    /// length produces infinities.
    #[inline(always)]
    pub fn normalize(self) -> Self {
        let len = self.length();
        Self::new(
            self.token,
            self.x / len,
            self.y / len,
            self.z / len,
            self.w / len,
        )
    }

    /// The conjugate: the inverse rotation when the quaternion is unit length.
    #[inline(always)]
    pub fn conjugate(self) -> Self {
        Self::new(self.token, -self.x, -self.y, -self.z, self.w)
    }

    /// The inverse rotation, which for unit quaternions is the
    /// [`conjugate`](Self::conjugate), as in glam.
    #[inline(always)]
    pub fn inverse(self) -> Self {
        self.conjugate()
    }

    /// The Hamilton product: the composition of the two rotations.
    #[inline(always)]
    pub fn mul_quat(self, rhs: Self) -> Self {
        let (x0, y0, z0, w0) = (self.x, self.y, self.z, self.w);
        let (x1, y1, z1, w1) = (rhs.x, rhs.y, rhs.z, rhs.w);
        Self::new(
            self.token,
            w0 * x1 + x0 * w1 + y0 * z1 - z0 * y1,
            w0 * y1 - x0 * z1 + y0 * w1 + z0 * x1,
            w0 * z1 + x0 * y1 - y0 * x1 + z0 * w1,
            w0 * w1 - x0 * x1 - y0 * y1 - z0 * z1,
        )
    }

    /// Rotate `rhs` by this quaternion, one rotation per lane.
    ///
    /// This is the sandwich product `q v q*` written the way glam writes it, with one cross
    /// product and three dot products. Following glam, the quaternion is assumed to be unit
    /// length; a non-unit quaternion scales the result by its squared length.
    #[inline(always)]
    pub fn mul_vec3(self, rhs: Vec3xN<S>) -> Vec3xN<S> {
        let b = Vec3xN::new(self.token, self.x, self.y, self.z);
        let b2 = b.dot(b);
        let w2_minus_b2 = self.w * self.w - b2;
        let two = S::f32s::splat(self.token, 2.0);

        scale3(rhs, w2_minus_b2) + scale3(b, rhs.dot(b) * two) + scale3(b.cross(rhs), self.w * two)
    }
}

impl<S: Simd> ExtractToken for QuatxN<S> {
    type S = S;
    #[inline(always)]
    fn token(&self) -> S {
        self.token
    }
}

impl<S: Simd> Add for QuatxN<S> {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.token,
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl<S: Simd> AddAssign for QuatxN<S> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<S: Simd> Sub for QuatxN<S> {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.token,
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl<S: Simd> SubAssign for QuatxN<S> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<S: Simd> Neg for QuatxN<S> {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self::new(self.token, -self.x, -self.y, -self.z, -self.w)
    }
}

impl<S: Simd> Mul for QuatxN<S> {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        self.mul_quat(rhs)
    }
}

impl<S: Simd> Mul<Vec3xN<S>> for QuatxN<S> {
    type Output = Vec3xN<S>;
    #[inline(always)]
    fn mul(self, rhs: Vec3xN<S>) -> Vec3xN<S> {
        self.mul_vec3(rhs)
    }
}
