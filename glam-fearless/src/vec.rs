//! Wide vector types (`Vec2xN`, `Vec3xN`, `Vec4xN`).
//!
//! Each component is a native-width `f32` SIMD vector (`S::f32s`) and each value
//! carries its zero-sized SIMD token, so operators need no explicit token
//! argument.

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use fearless_simd::prelude::*;
use fearless_simd::{ExtractToken, Simd, SimdFrom};

use crate::bvec::{BVec2xN, BVec3xN, BVec4xN};

/// Element-wise arithmetic and token plumbing shared by all wide vectors.
macro_rules! impl_wide_ops {
    ($name:ident, ($($field:ident),+)) => {
        impl<S: Simd> Add for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                Self { $($field: self.$field + rhs.$field,)+ token: self.token }
            }
        }

        impl<S: Simd> AddAssign for $name<S> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl<S: Simd> Sub for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                Self { $($field: self.$field - rhs.$field,)+ token: self.token }
            }
        }

        impl<S: Simd> SubAssign for $name<S> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl<S: Simd> Mul for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self {
                Self { $($field: self.$field * rhs.$field,)+ token: self.token }
            }
        }

        impl<S: Simd> MulAssign for $name<S> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl<S: Simd> Div for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: Self) -> Self {
                Self { $($field: self.$field / rhs.$field,)+ token: self.token }
            }
        }

        impl<S: Simd> DivAssign for $name<S> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }

        impl<S: Simd> Mul<f32> for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: f32) -> Self {
                Self { $($field: self.$field * rhs,)+ token: self.token }
            }
        }

        impl<S: Simd> Mul<$name<S>> for f32 {
            type Output = $name<S>;
            #[inline(always)]
            fn mul(self, rhs: $name<S>) -> $name<S> {
                rhs * self
            }
        }

        impl<S: Simd> Div<f32> for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn div(self, rhs: f32) -> Self {
                Self { $($field: self.$field / rhs,)+ token: self.token }
            }
        }

        impl<S: Simd> Neg for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn neg(self) -> Self {
                Self { $($field: -self.$field,)+ token: self.token }
            }
        }

        impl<S: Simd> ExtractToken for $name<S> {
            type S = S;
            #[inline(always)]
            fn token(&self) -> S {
                self.token
            }
        }

        /// A scalar is broadcast to every lane of every component.
        impl<S: Simd> SimdFrom<f32, S> for $name<S> {
            #[inline(always)]
            fn simd_from(simd: S, value: f32) -> Self {
                Self::splat(simd, value)
            }
        }
    };
}

/// A 2-dimensional wide vector.
#[derive(Clone, Copy, Debug)]
pub struct Vec2xN<S: Simd> {
    /// The X component.
    pub x: S::f32s,
    /// The Y component.
    pub y: S::f32s,
    token: S,
}

/// A 3-dimensional wide vector.
#[derive(Clone, Copy, Debug)]
pub struct Vec3xN<S: Simd> {
    /// The X component.
    pub x: S::f32s,
    /// The Y component.
    pub y: S::f32s,
    /// The Z component.
    pub z: S::f32s,
    token: S,
}

/// A 4-dimensional wide vector.
#[derive(Clone, Copy, Debug)]
pub struct Vec4xN<S: Simd> {
    /// The X component.
    pub x: S::f32s,
    /// The Y component.
    pub y: S::f32s,
    /// The Z component.
    pub z: S::f32s,
    /// The W component.
    pub w: S::f32s,
    token: S,
}

macro_rules! wide_vec_common {
    ($name:ident, $bvec:ident, $glam_vec:ty, $splat:ident, ($($field:ident),+)) => {
        impl<S: Simd> $name<S> {
            /// The SIMD token this value carries.
            #[inline(always)]
            pub fn token(&self) -> S {
                self.token
            }

            /// Number of `f32` lanes per component: the width this level processes at
            /// once, equal to `S::f32s::LEN`.
            pub const LANES: usize = <S::f32s as SimdBase<S>>::LEN;

            /// Construct one lane-vector per component.
            #[inline(always)]
            pub fn new(simd: S, $($field: S::f32s),+) -> Self {
                Self { $($field,)+ token: simd }
            }

            /// Construct from one `f32` per component, splatted across the lanes.
            #[inline(always)]
            pub fn $splat(simd: S, $($field: f32),+) -> Self {
                Self::new(simd, $(S::f32s::splat(simd, $field)),+)
            }

            /// Construct from a narrow glam vector, splatting each component across the
            /// lanes.
            ///
            /// Takes anything convertible to the matching glam vector, so `glam::Vec3`
            /// and `[f32; 3]` both work.
            #[inline(always)]
            pub fn splat_vec(simd: S, value: impl Into<$glam_vec>) -> Self {
                let value: $glam_vec = value.into();
                Self::$splat(simd, $(value.$field),+)
            }

            /// Construct with every lane of every component set to `value`.
            #[inline(always)]
            pub fn splat(simd: S, value: f32) -> Self {
                Self::$splat(simd, $(replace_expr!($field, value)),+)
            }

            /// All lanes of all components are zero.
            #[inline(always)]
            pub fn zero(simd: S) -> Self {
                Self::splat(simd, 0.0)
            }

            /// Construct from one lane slice per component.
            ///
            /// Each slice must be exactly `S::f32s::LEN` long.
            #[inline(always)]
            pub fn from_slice(simd: S, $($field: &[f32]),+) -> Self {
                Self::new(simd, $(S::f32s::from_slice(simd, $field)),+)
            }

            /// Component-wise minimum.
            #[inline(always)]
            pub fn min(self, rhs: Self) -> Self {
                Self::new(self.token, $(self.$field.min(rhs.$field)),+)
            }

            /// Component-wise maximum.
            #[inline(always)]
            pub fn max(self, rhs: Self) -> Self {
                Self::new(self.token, $(self.$field.max(rhs.$field)),+)
            }

            /// Component-wise absolute value.
            #[inline(always)]
            pub fn abs(self) -> Self {
                Self::new(self.token, $(self.$field.abs()),+)
            }

            /// Component-wise `self > rhs`.
            #[inline(always)]
            pub fn cmp_gt(self, rhs: Self) -> $bvec<S> {
                $bvec::new(self.token, $(self.$field.simd_gt(rhs.$field)),+)
            }

            /// Component-wise `self >= rhs`.
            #[inline(always)]
            pub fn cmp_ge(self, rhs: Self) -> $bvec<S> {
                $bvec::new(self.token, $(self.$field.simd_ge(rhs.$field)),+)
            }

            /// Component-wise `self < rhs`.
            #[inline(always)]
            pub fn cmp_lt(self, rhs: Self) -> $bvec<S> {
                $bvec::new(self.token, $(self.$field.simd_lt(rhs.$field)),+)
            }

            /// Component-wise `self <= rhs`.
            #[inline(always)]
            pub fn cmp_le(self, rhs: Self) -> $bvec<S> {
                $bvec::new(self.token, $(self.$field.simd_le(rhs.$field)),+)
            }

            /// Component-wise `self == rhs`.
            #[inline(always)]
            pub fn cmp_eq(self, rhs: Self) -> $bvec<S> {
                $bvec::new(self.token, $(self.$field.simd_eq(rhs.$field)),+)
            }

            /// Component-wise `self != rhs`.
            #[inline(always)]
            pub fn cmp_ne(self, rhs: Self) -> $bvec<S> {
                $bvec::new(self.token, $(self.$field.simd_ne(rhs.$field)),+)
            }

            /// Component-wise select, in glam's static form: `if_true` where `mask` is
            /// true, `if_false` where it is false.
            ///
            /// This is the only spelling: glam puts `select` on the vector, not on the
            /// mask type, and a mask-owned spelling would be an invented duplicate.
            #[inline(always)]
            pub fn select(mask: $bvec<S>, if_true: Self, if_false: Self) -> Self {
                Self::new(
                    if_true.token,
                    $(mask.$field.select(if_true.$field, if_false.$field)),+
                )
            }
        }
    };
}

/// Ops that collapse the component axis, so the first component is named
/// separately to avoid counting it twice in the macro expansion.
macro_rules! impl_horizontal_ops {
    ($name:ident, $first:ident $(, $rest:ident)+) => {
        impl<S: Simd> $name<S> {
            /// Dot product, summed across components; one result per lane.
            #[inline(always)]
            pub fn dot(self, rhs: Self) -> S::f32s {
                self.$first * rhs.$first $(+ self.$rest * rhs.$rest)+
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
            /// Divides by the length across lanes, rather than multiplying by a reciprocal:
            /// one rounding instead of two, which is what glam's SIMD path and nalgebra do
            /// (glam's scalar path uses a reciprocal, ultraviolet uses one too). A lane of
            /// zero length produces infinities, as it does in glam.
            #[inline(always)]
            pub fn normalize(self) -> Self {
                let len = self.length();
                Self::new(
                    self.token,
                    self.$first / len,
                    $(self.$rest / len),+
                )
            }
        }
    };
}

// `Self::splat(simd, value, value, …)`: repeats a token for each component without
// naming it once per field in the macro body.
macro_rules! replace_expr {
    ($_t:tt, $sub:expr) => {
        $sub
    };
}

wide_vec_common!(Vec2xN, BVec2xN, glam::Vec2, splat_xy, (x, y));
wide_vec_common!(Vec3xN, BVec3xN, glam::Vec3, splat_xyz, (x, y, z));
wide_vec_common!(Vec4xN, BVec4xN, glam::Vec4, splat_xyzw, (x, y, z, w));

impl_horizontal_ops!(Vec2xN, x, y);
impl_horizontal_ops!(Vec3xN, x, y, z);
impl_horizontal_ops!(Vec4xN, x, y, z, w);

impl_wide_ops!(Vec2xN, (x, y));
impl_wide_ops!(Vec3xN, (x, y, z));
impl_wide_ops!(Vec4xN, (x, y, z, w));

impl<S: Simd> Vec2xN<S> {
    /// Perpendicular dot product: `self.x * rhs.y - self.y * rhs.x`.
    #[inline(always)]
    pub fn perp_dot(self, rhs: Self) -> S::f32s {
        self.x * rhs.y - self.y * rhs.x
    }
}

impl<S: Simd> Vec3xN<S> {
    /// Cross product.
    #[inline(always)]
    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.token,
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
}

impl<S: Simd> Vec4xN<S> {
    /// Truncate to a `Vec3xN`, dropping W.
    #[inline(always)]
    pub fn truncate(self) -> Vec3xN<S> {
        Vec3xN::new(self.token, self.x, self.y, self.z)
    }
}
