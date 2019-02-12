#![allow(dead_code)]
use crate::vmath::Vec3;
use rand::{Rng, XorShiftRng};

#[derive(Debug)]
pub struct Perlin {
    ranfloat: Vec<f32>,
    perm_x: Vec<u32>,
    perm_y: Vec<u32>,
    perm_z: Vec<u32>,
}

impl Perlin {
    fn generate(rng: &mut XorShiftRng) -> Vec<f32> {
        let mut ranfloat = vec![0.0; 256];
        for f in ranfloat.iter_mut() {
            *f = rng.next_f32();
        }
        ranfloat
    }

    fn permute(rng: &mut XorShiftRng, perm: &mut Vec<u32>) {
        for i in (0..perm.len()).rev() {
            let target = (rng.next_f32() * (i + 1) as f32).floor() as usize;
            perm.swap(i, target);
        }
    }

    fn generate_perm(rng: &mut XorShiftRng) -> Vec<u32> {
        let mut perm = vec![0; 256];
        for (i, p) in perm.iter_mut().enumerate() {
            *p = i as u32;
        }
        Perlin::permute(rng, &mut perm);
        perm
    }

    pub fn new(rng: &mut XorShiftRng) -> Perlin {
        Perlin {
            ranfloat: Perlin::generate(rng),
            perm_x: Perlin::generate_perm(rng),
            perm_y: Perlin::generate_perm(rng),
            perm_z: Perlin::generate_perm(rng),
        }
    }

    pub fn noise(&self, p: Vec3) -> f32 {
        let x = p.get_x();
        let y = p.get_y();
        let z = p.get_z();
        // let u = x - x.floor();
        // let v = y - y.floor();
        // let w = z - z.floor();
        let i = self.perm_x[(4.0 * x) as usize & 255] as usize;
        let j = self.perm_y[(4.0 * y) as usize & 255] as usize;
        let k = self.perm_z[(4.0 * z) as usize & 255] as usize;
        self.ranfloat[i ^ j ^ k]
    }
}
