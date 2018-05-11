use std::mem;
use std::num::Wrapping;

pub trait Rng {
    fn next_u32(&mut self) -> u32;
    fn next_f32(&mut self) -> f32;
}

pub trait SeedableRng: Rng {
    fn from_seed(seed: u32) -> Self;
}

pub struct XorShift32Rng {
    state: Wrapping<u32>
}

impl SeedableRng for XorShift32Rng {
    #[inline]
    fn from_seed(seed: u32) -> XorShift32Rng {
        assert!(seed != 0);
        XorShift32Rng {
            state: Wrapping(seed)
        }
    }
}

impl Rng for XorShift32Rng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x.0
    }

    #[inline]
    fn next_f32(&mut self) -> f32 {
        const UPPER_MASK: u32 = 0x3F800000;
        const LOWER_MASK: u32 = 0x7FFFFF;
        let tmp = UPPER_MASK | (self.next_u32() & LOWER_MASK);
        let result: f32 = unsafe { mem::transmute(tmp) };
        result - 1.0
        // const MASK: u32 = 0xFFFFFF;
        // let tmp = self.next_u32() & MASK;
        // let result: f32 = unsafe { mem::transmute(tmp) };
        // result / 16777216.0
        // let r = unsafe { transmute::<u32, f32>(self.next_u32() & 0xFFFFFF) };
        // r / 16777216.0
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng, XorShift32Rng};
    #[test]
    fn u32_test() {
        let mut rng = XorShift32Rng::from_seed(10);
        assert_eq!(2703690, rng.next_u32());
        assert_eq!(671267850, rng.next_u32());
        assert_eq!(2415639211, rng.next_u32());
        assert_eq!(3984226684, rng.next_u32());
    }
}
