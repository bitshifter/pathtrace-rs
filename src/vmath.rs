/*
 * Copyright (c) 2012-2013 Graham Sellers
 * Copyright (c) 2014 Cameron Hart
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */
#![allow(dead_code)]


#[cfg(target_feature = "sse2")]
pub use self::sse2::*;

#[cfg(not(target_feature = "sse2"))]
pub use self::scalar::*;

mod sse2 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    use std::f32;
    use std::fmt;
    use std::ops::*;

    #[derive(Clone, Copy, Debug)]
    #[repr(C)]
    pub struct Vec3(__m128);

    macro_rules! _mm_shuffle {
        ($z:expr, $y:expr, $x:expr, $w:expr) => {
            ($z << 6) | ($y << 4) | ($x << 2) | $w
        };
    }

    #[inline]
    pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new(x, y, z)
    }

    impl Vec3 {
        #[inline]
        pub fn zero() -> Vec3 {
            unsafe { Vec3(_mm_set1_ps(0.0)) }
        }

        #[inline]
        pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
            unsafe { Vec3(_mm_set_ps(z, z, y, x)) }
        }

        #[inline]
        pub fn get_x(self) -> f32 {
            unsafe { _mm_cvtss_f32(self.0) }
        }

        #[inline]
        pub fn get_y(self) -> f32 {
            unsafe { _mm_cvtss_f32(_mm_shuffle_ps(self.0, self.0, _mm_shuffle!(1, 1, 1, 1))) }
        }

        #[inline]
        pub fn get_z(self) -> f32 {
            unsafe { _mm_cvtss_f32(_mm_shuffle_ps(self.0, self.0, _mm_shuffle!(2, 2, 2, 2))) }
        }

        #[inline]
        pub fn yzx(self) -> Vec3 {
            unsafe { Vec3(_mm_shuffle_ps(self.0, self.0, _mm_shuffle!(0, 0, 2, 1))) }
        }

        #[inline]
        pub fn zxy(self) -> Vec3 {
            unsafe { Vec3(_mm_shuffle_ps(self.0, self.0, _mm_shuffle!(1, 1, 0, 2))) }
        }

        #[inline]
        pub fn set_x(&mut self, x: f32) {
            unsafe {
                self.0 = _mm_move_ss(self.0, _mm_set_ss(x));
            }
        }

        #[inline]
        pub fn set_y(&mut self, y: f32) {
            unsafe {
                let mut t = _mm_move_ss(self.0, _mm_set_ss(y));
                t = _mm_shuffle_ps(t, t, _mm_shuffle!(3, 2, 0, 0));
                self.0 = _mm_move_ss(t, self.0);
            }
        }

        #[inline]
        pub fn set_z(&mut self, z: f32) {
            unsafe {
                let mut t = _mm_move_ss(self.0, _mm_set_ss(z));
                t = _mm_shuffle_ps(t, t, _mm_shuffle!(3, 0, 1, 0));
                self.0 = _mm_move_ss(t, self.0);
            }
        }

        #[inline]
        pub fn sum(self) -> f32 {
            self.get_x() + self.get_y() + self.get_z()
        }

        #[inline]
        pub fn dot(self, rhs: Vec3) -> f32 {
            (self * rhs).sum()
        }

        #[inline]
        pub fn cross(self, rhs: Vec3) -> Vec3 {
            // x  <-  a.y*b.z - a.z*b.y
            // y  <-  a.z*b.x - a.x*b.z
            // z  <-  a.x*b.y - a.y*b.x
            // We can save a shuffle by grouping it in this wacky order:
            (self.zxy() * rhs - self * rhs.zxy()).zxy()
        }

        #[inline]
        pub fn length(self) -> f32 {
            self.dot(self).sqrt()
        }

        #[inline]
        pub fn length_squared(self) -> f32 {
            self.dot(self)
        }

        #[inline]
        pub fn normalize(self) -> Vec3 {
            let inv_length = 1.0 / self.dot(self).sqrt();
            self * inv_length
        }
    }

    impl fmt::Display for Vec3 {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "[{}, {}, {}]", self.get_x(), self.get_y(), self.get_z())
        }
    }

    impl Div<f32> for Vec3 {
        type Output = Vec3;
        #[inline]
        fn div(self, rhs: f32) -> Vec3 {
            unsafe { Vec3(_mm_div_ps(self.0, _mm_set1_ps(rhs))) }
        }
    }

    impl DivAssign<f32> for Vec3 {
        #[inline]
        fn div_assign(&mut self, rhs: f32) {
            unsafe { self.0 = _mm_div_ps(self.0, _mm_set1_ps(rhs)) }
        }
    }

    impl Mul<Vec3> for Vec3 {
        type Output = Vec3;
        #[inline]
        fn mul(self, rhs: Vec3) -> Vec3 {
            unsafe { Vec3(_mm_mul_ps(self.0, rhs.0)) }
        }
    }

    impl MulAssign<Vec3> for Vec3 {
        #[inline]
        fn mul_assign(&mut self, rhs: Vec3) {
            unsafe {
                self.0 = _mm_mul_ps(self.0, rhs.0);
            }
        }
    }

    impl Mul<f32> for Vec3 {
        type Output = Vec3;
        #[inline]
        fn mul(self, rhs: f32) -> Vec3 {
            unsafe { Vec3(_mm_mul_ps(self.0, _mm_set1_ps(rhs))) }
        }
    }

    impl MulAssign<f32> for Vec3 {
        #[inline]
        fn mul_assign(&mut self, rhs: f32) {
            unsafe { self.0 = _mm_mul_ps(self.0, _mm_set1_ps(rhs)) }
        }
    }

    impl Mul<Vec3> for f32 {
        type Output = Vec3;
        #[inline]
        fn mul(self, rhs: Vec3) -> Vec3 {
            unsafe { Vec3(_mm_mul_ps(_mm_set1_ps(self), rhs.0)) }
        }
    }

    impl Add for Vec3 {
        type Output = Vec3;
        #[inline]
        fn add(self, rhs: Vec3) -> Vec3 {
            unsafe { Vec3(_mm_add_ps(self.0, rhs.0)) }
        }
    }

    impl AddAssign for Vec3 {
        #[inline]
        fn add_assign(&mut self, rhs: Vec3) {
            unsafe { self.0 = _mm_add_ps(self.0, rhs.0) }
        }
    }

    impl Sub for Vec3 {
        type Output = Vec3;
        #[inline]
        fn sub(self, rhs: Vec3) -> Vec3 {
            unsafe { Vec3(_mm_sub_ps(self.0, rhs.0)) }
        }
    }

    impl SubAssign for Vec3 {
        #[inline]
        fn sub_assign(&mut self, rhs: Vec3) {
            unsafe { self.0 = _mm_sub_ps(self.0, rhs.0) }
        }
    }

    impl Neg for Vec3 {
        type Output = Vec3;
        #[inline]
        fn neg(self) -> Vec3 {
            unsafe { Vec3(_mm_sub_ps(_mm_set1_ps(0.0), self.0)) }
        }
    }
}

mod scalar {
    use std::f32;
    use std::fmt;
    use std::ops::*;
    #[derive(Clone, Copy, Debug)]
    #[repr(C)]
    pub struct Vec3(f32, f32, f32);

    #[inline]
    pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3(x, y, z)
    }

    impl Vec3 {
        #[inline]
        pub fn zero() -> Vec3 {
            Vec3(0.0, 0.0, 0.0)
        }

        #[inline]
        pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
            Vec3(x, y, z)
        }

        #[inline]
        pub fn get_x(self) -> f32 {
            self.0
        }

        #[inline]
        pub fn get_y(self) -> f32 {
            self.1
        }

        #[inline]
        pub fn get_z(self) -> f32 {
            self.2
        }

        #[inline]
        pub fn dot(self, rhs: Vec3) -> f32 {
            (self.0 * rhs.0) + (self.1 * rhs.1) + (self.2 * rhs.2)
        }

        #[inline]
        pub fn cross(self, rhs: Vec3) -> Vec3 {
            Vec3(
                self.1 * rhs.2 - rhs.1 * self.2,
                self.2 * rhs.0 - rhs.2 * self.0,
                self.0 * rhs.1 - rhs.0 * self.1,
            )
        }

        #[inline]
        pub fn length(self) -> f32 {
            self.dot(self).sqrt()
        }

        #[inline]
        pub fn length_squared(self) -> f32 {
            self.dot(self)
        }

        #[inline]
        pub fn normalize(self) -> Vec3 {
            let inv_length = 1.0 / self.dot(self).sqrt();
            self * inv_length
        }
    }

    impl fmt::Display for Vec3 {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "[{}, {}, {}]", self.0, self.1, self.2)
        }
    }

    impl Div<f32> for Vec3 {
        type Output = Vec3;
        #[inline]
        fn div(self, rhs: f32) -> Vec3 {
            Vec3(self.0 / rhs, self.1 / rhs, self.2 / rhs)
        }
    }

    impl DivAssign<f32> for Vec3 {
        #[inline]
        fn div_assign(&mut self, rhs: f32) {
            *self = Vec3(self.0 / rhs, self.1 / rhs, self.2 / rhs)
        }
    }

    impl Mul<Vec3> for Vec3 {
        type Output = Vec3;
        #[inline]
        fn mul(self, rhs: Vec3) -> Vec3 {
            Vec3(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
        }
    }

    impl MulAssign<Vec3> for Vec3 {
        #[inline]
        fn mul_assign(&mut self, rhs: Vec3) {
            *self = Vec3(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
        }
    }

    impl Mul<f32> for Vec3 {
        type Output = Vec3;
        #[inline]
        fn mul(self, rhs: f32) -> Vec3 {
            Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
        }
    }

    impl MulAssign<f32> for Vec3 {
        #[inline]
        fn mul_assign(&mut self, rhs: f32) {
            *self = Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
        }
    }

    impl Mul<Vec3> for f32 {
        type Output = Vec3;
        #[inline]
        fn mul(self, rhs: Vec3) -> Vec3 {
            Vec3(self * rhs.0, self * rhs.1, self * rhs.2)
        }
    }

    impl Add for Vec3 {
        type Output = Vec3;
        #[inline]
        fn add(self, rhs: Vec3) -> Vec3 {
            Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
        }
    }

    impl AddAssign for Vec3 {
        #[inline]
        fn add_assign(&mut self, rhs: Vec3) {
            *self = Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
        }
    }

    impl Sub for Vec3 {
        type Output = Vec3;
        #[inline]
        fn sub(self, rhs: Vec3) -> Vec3 {
            Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
        }
    }

    impl SubAssign for Vec3 {
        #[inline]
        fn sub_assign(&mut self, rhs: Vec3) {
            *self = Vec3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
        }
    }

    impl Neg for Vec3 {
        type Output = Vec3;
        #[inline]
        fn neg(self) -> Vec3 {
            Vec3(-self.0, -self.1, -self.2)
        }
    }
}
