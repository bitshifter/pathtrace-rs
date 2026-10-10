//! Wide boolean (mask) vector types: what component-wise comparisons on
//! [`Vec2xN`](crate::Vec2xN)/[`Vec3xN`](crate::Vec3xN)/[`Vec4xN`](crate::Vec4xN) return.
//!
//! Each component holds one lane mask (`S::mask32s`), mirroring the shape of glam's
//! `BVec2`/`BVec3`/`BVec4` with lanes instead of single booleans. The types support
//! component-wise `&`/`|`/`!`, and selection is spelled on the vector
//! (`Vec3xN::select(mask, if_true, if_false)`), as it is in glam.
//!
//! Reductions are not part of this type: use the lane masks directly — `mask.any_true()`,
//! `mask.all_true()`, `mask.to_bitmask()` — from the `fearless_simd` prelude.

use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use fearless_simd::prelude::*;
use fearless_simd::{ExtractToken, Simd};

/// Repeats `$sub` once per field name, so a macro repetition can be driven by the
/// field list even when the expanded expression does not mention the field.
macro_rules! replace_expr {
    ($_t:tt, $sub:expr) => {
        $sub
    };
}

macro_rules! wide_bvec {
    ($name:ident, ($($field:ident),+)) => {
        /// A wide boolean vector; each component is one lane mask.
        #[derive(Clone, Copy, Debug)]
        pub struct $name<S: Simd> {
            $(pub $field: S::mask32s,)+
            token: S,
        }

        impl<S: Simd> $name<S> {
            /// Construct from one lane mask per component.
            #[inline(always)]
            pub fn new(simd: S, $($field: S::mask32s),+) -> Self {
                Self { $($field,)+ token: simd }
            }

            /// Construct with every lane of every component set to `value`.
            #[inline(always)]
            pub fn splat(simd: S, value: bool) -> Self {
                Self::new(simd, $(replace_expr!($field, S::mask32s::splat(simd, value))),+)
            }

            /// The SIMD token this value carries.
            #[inline(always)]
            pub fn token(&self) -> S {
                self.token
            }

        }

        impl<S: Simd> ExtractToken for $name<S> {
            type S = S;
            #[inline(always)]
            fn token(&self) -> S {
                self.token
            }
        }

        impl<S: Simd> BitAnd for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self {
                Self { $($field: self.$field & rhs.$field,)+ token: self.token }
            }
        }

        impl<S: Simd> BitAndAssign for $name<S> {
            #[inline(always)]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs;
            }
        }

        impl<S: Simd> BitOr for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self {
                Self { $($field: self.$field | rhs.$field,)+ token: self.token }
            }
        }

        impl<S: Simd> BitOrAssign for $name<S> {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs;
            }
        }

        impl<S: Simd> Not for $name<S> {
            type Output = Self;
            #[inline(always)]
            fn not(self) -> Self {
                Self { $($field: !self.$field,)+ token: self.token }
            }
        }
    };
}

wide_bvec!(BVec2xN, (x, y));
wide_bvec!(BVec3xN, (x, y, z));
wide_bvec!(BVec4xN, (x, y, z, w));
