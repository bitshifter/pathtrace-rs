//! Wide affine transform: a 3x3 linear part plus a translation, one transform per lane.
//!
//! Composed as glam composes it, from [`Mat3xN`] and [`Vec3xN`]. Point and vector transforms reuse
//! the matrix maths, so an affine is a thin wrapper rather than a second implementation.

use core::ops::Mul;

use fearless_simd::{ExtractToken, Simd};

use crate::mat3::Mat3xN;
use crate::vec::Vec3xN;

/// A wide affine transform; `matrix3` is the linear part and `translation` the offset.
#[derive(Clone, Copy, Debug)]
pub struct Affine3xN<S: Simd> {
    /// The linear part.
    pub matrix3: Mat3xN<S>,
    /// The translation.
    pub translation: Vec3xN<S>,
    token: S,
}

impl<S: Simd> Affine3xN<S> {
    /// The SIMD token this value carries.
    #[inline(always)]
    pub fn token(&self) -> S {
        self.token
    }

    /// Construct from a linear part and a translation.
    #[inline(always)]
    pub fn from_mat3_translation(simd: S, matrix3: Mat3xN<S>, translation: Vec3xN<S>) -> Self {
        Self {
            matrix3,
            translation,
            token: simd,
        }
    }

    /// The identity transform in every lane.
    #[inline(always)]
    pub fn identity(simd: S) -> Self {
        Self::from_mat3_translation(simd, Mat3xN::identity(simd), Vec3xN::zero(simd))
    }

    /// All zeros.
    #[inline(always)]
    pub fn zero(simd: S) -> Self {
        Self::from_mat3_translation(simd, Mat3xN::zero(simd), Vec3xN::zero(simd))
    }

    /// Construct from a narrow glam affine transform, one transform broadcast to every lane.
    #[inline(always)]
    pub fn splat_affine3(simd: S, value: glam::Affine3) -> Self {
        Self::from_mat3_translation(
            simd,
            Mat3xN::splat_mat3(simd, value.matrix3),
            Vec3xN::splat_vec3(simd, value.translation),
        )
    }

    /// Transform a point: the linear part applied, then the translation.
    #[inline(always)]
    pub fn transform_point3(self, rhs: Vec3xN<S>) -> Vec3xN<S> {
        self.matrix3.mul_vec3(rhs) + self.translation
    }

    /// Transform a vector: the linear part only, no translation.
    #[inline(always)]
    pub fn transform_vector3(self, rhs: Vec3xN<S>) -> Vec3xN<S> {
        self.matrix3.mul_vec3(rhs)
    }

    /// Compose two transforms: `self` applied after `rhs`.
    ///
    /// glam names a matrix product after its right hand side, as [`Mat3xN::mul_mat3`] does, but
    /// gives `Affine3` no named product, so this follows the matrix naming for symmetry.
    #[inline(always)]
    pub fn mul_affine3(self, rhs: Self) -> Self {
        Self::from_mat3_translation(
            self.token,
            self.matrix3 * rhs.matrix3,
            self.matrix3.mul_vec3(rhs.translation) + self.translation,
        )
    }

    /// The inverse transform.
    ///
    /// A lane whose linear part has a zero determinant produces infinities or NaN.
    #[inline(always)]
    pub fn inverse(self) -> Self {
        let inv = self.matrix3.inverse();
        Self::from_mat3_translation(self.token, inv, -(inv.mul_vec3(self.translation)))
    }
}

impl<S: Simd> ExtractToken for Affine3xN<S> {
    type S = S;
    #[inline(always)]
    fn token(&self) -> S {
        self.token
    }
}

impl<S: Simd> Mul for Affine3xN<S> {
    type Output = Self;
    /// Composes two transforms, as [`mul_affine3`](Self::mul_affine3) does.
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        self.mul_affine3(rhs)
    }
}

impl<S: Simd> Mul<Vec3xN<S>> for Affine3xN<S> {
    type Output = Vec3xN<S>;
    /// Transforms a point, as glam's `Affine3 * Vec3` does.
    #[inline(always)]
    fn mul(self, rhs: Vec3xN<S>) -> Vec3xN<S> {
        self.transform_point3(rhs)
    }
}
