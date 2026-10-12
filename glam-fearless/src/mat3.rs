//! Wide 3x3 matrix type: one matrix per lane.
//!
//! Column-major, with glam's column names, so `mat * vec` is a combination of the columns (three
//! multiplies and two adds per output component, with no horizontal sums) and `transpose` is
//! regrouping of lane vectors rather than shuffles.

use core::ops::Mul;

use fearless_simd::prelude::*;
use fearless_simd::{ExtractToken, Simd};

use crate::vec::{Vec3xN, scale3};

/// A wide 3x3 matrix; `x_axis`/`y_axis`/`z_axis` are the columns, each a [`Vec3xN`].
#[derive(Clone, Copy, Debug)]
pub struct Mat3xN<S: Simd> {
    /// The first column.
    pub x_axis: Vec3xN<S>,
    /// The second column.
    pub y_axis: Vec3xN<S>,
    /// The third column.
    pub z_axis: Vec3xN<S>,
    token: S,
}

impl<S: Simd> Mat3xN<S> {
    /// The SIMD token this value carries.
    #[inline(always)]
    pub fn token(&self) -> S {
        self.token
    }

    /// Construct from three columns.
    #[inline(always)]
    pub fn from_cols(simd: S, x_axis: Vec3xN<S>, y_axis: Vec3xN<S>, z_axis: Vec3xN<S>) -> Self {
        Self {
            x_axis,
            y_axis,
            z_axis,
            token: simd,
        }
    }

    /// The columns, as an array.
    #[inline(always)]
    pub fn cols(self) -> [Vec3xN<S>; 3] {
        [self.x_axis, self.y_axis, self.z_axis]
    }

    /// All zeros.
    #[inline(always)]
    pub fn zero(simd: S) -> Self {
        Self::from_cols(
            simd,
            Vec3xN::zero(simd),
            Vec3xN::zero(simd),
            Vec3xN::zero(simd),
        )
    }

    /// The identity matrix in every lane.
    #[inline(always)]
    pub fn identity(simd: S) -> Self {
        let zero = S::f32s::splat(simd, 0.0);
        let one = S::f32s::splat(simd, 1.0);
        Self::from_cols(
            simd,
            Vec3xN::new(simd, one, zero, zero),
            Vec3xN::new(simd, zero, one, zero),
            Vec3xN::new(simd, zero, zero, one),
        )
    }

    /// Construct from a narrow glam matrix, one matrix broadcast to every lane.
    #[inline(always)]
    pub fn splat_mat3(simd: S, value: glam::Mat3) -> Self {
        Self::from_cols(
            simd,
            Vec3xN::splat_vec3(simd, value.x_axis),
            Vec3xN::splat_vec3(simd, value.y_axis),
            Vec3xN::splat_vec3(simd, value.z_axis),
        )
    }

    /// Swap rows and columns.
    ///
    /// In this representation that is regrouping lane vectors, no shuffles.
    #[inline(always)]
    pub fn transpose(self) -> Self {
        Self::from_cols(
            self.token,
            Vec3xN::new(self.token, self.x_axis.x, self.y_axis.x, self.z_axis.x),
            Vec3xN::new(self.token, self.x_axis.y, self.y_axis.y, self.z_axis.y),
            Vec3xN::new(self.token, self.x_axis.z, self.y_axis.z, self.z_axis.z),
        )
    }

    /// Multiply a vector: a combination of the columns, one result per lane.
    #[inline(always)]
    pub fn mul_vec3(self, rhs: Vec3xN<S>) -> Vec3xN<S> {
        scale3(self.x_axis, rhs.x) + scale3(self.y_axis, rhs.y) + scale3(self.z_axis, rhs.z)
    }

    /// Multiply two matrices: the composition, one result per lane.
    ///
    /// Each column of the result is `self` applied to the matching column of `rhs`, so
    /// `m1.mul_mat3(m2).mul_vec3(v)` is `m1.mul_vec3(m2.mul_vec3(v))`.
    #[inline(always)]
    pub fn mul_mat3(self, rhs: Self) -> Self {
        Self::from_cols(
            self.token,
            self.mul_vec3(rhs.x_axis),
            self.mul_vec3(rhs.y_axis),
            self.mul_vec3(rhs.z_axis),
        )
    }

    /// The determinant, one result per lane.
    #[inline(always)]
    pub fn determinant(self) -> S::f32s {
        self.x_axis.dot(self.y_axis.cross(self.z_axis))
    }

    /// The inverse, one result per lane.
    ///
    /// The rows of the inverse are the cross products of the columns over the determinant. A
    /// lane with a zero determinant produces infinities or NaN, as it does in glam.
    #[inline(always)]
    pub fn inverse(self) -> Self {
        let r0 = self.y_axis.cross(self.z_axis);
        let r1 = self.z_axis.cross(self.x_axis);
        let r2 = self.x_axis.cross(self.y_axis);
        let inv_det = S::f32s::splat(self.token, 1.0) / self.x_axis.dot(r0);
        Self::from_cols(
            self.token,
            Vec3xN::new(self.token, r0.x * inv_det, r1.x * inv_det, r2.x * inv_det),
            Vec3xN::new(self.token, r0.y * inv_det, r1.y * inv_det, r2.y * inv_det),
            Vec3xN::new(self.token, r0.z * inv_det, r1.z * inv_det, r2.z * inv_det),
        )
    }
}

impl<S: Simd> ExtractToken for Mat3xN<S> {
    type S = S;
    #[inline(always)]
    fn token(&self) -> S {
        self.token
    }
}

impl<S: Simd> Mul<Vec3xN<S>> for Mat3xN<S> {
    type Output = Vec3xN<S>;
    /// Multiplies the vector, with no translation: see [`mul_vec3`](Self::mul_vec3). Use
    /// [`Affine3xN`](crate::Affine3xN) to transform a point.
    #[inline(always)]
    fn mul(self, rhs: Vec3xN<S>) -> Vec3xN<S> {
        self.mul_vec3(rhs)
    }
}

impl<S: Simd> Mul for Mat3xN<S> {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        self.mul_mat3(rhs)
    }
}
