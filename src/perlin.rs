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

#[inline]
fn trilinear_interpolate(c: [[[f32; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
    let mut accum = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                accum += (i as f32 * u + (1.0 - i as f32) * (1.0 - u))
                    * (j as f32 * v + (1.0 - j as f32) * (1.0 - v))
                    * (k as f32 * w + (1.0 - k as f32) * (1.0 - w))
                    * c[i][j][k];
            }
        }
    }
    accum
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
        let mut u = x - x.floor();
        let mut v = y - y.floor();
        let mut w = z - z.floor();
        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);
        let i = x.floor() as usize;
        let j = y.floor() as usize;
        let k = z.floor() as usize;
        let mut c = [[[0.0; 2]; 2]; 2];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ranfloat[(self.perm_x[(i + di) & 255]
                        ^ self.perm_y[(j + dj) & 255]
                        ^ self.perm_z[(k + dk) & 255])
                        as usize]
                }
            }
        }
        trilinear_interpolate(c, u, v, w)
    }
}
