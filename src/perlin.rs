#![allow(dead_code)]
use crate::vmath::{vec3, Vec3};
use rand::{Rng, XorShiftRng};

#[derive(Debug)]
pub struct Perlin {
    randvec: Vec<Vec3>,
    perm_x: Vec<u32>,
    perm_y: Vec<u32>,
    perm_z: Vec<u32>,
}

impl Perlin {
    fn generate(rng: &mut XorShiftRng) -> Vec<Vec3> {
        let mut randvec = vec![Vec3::zero(); 256];
        for v in randvec.iter_mut() {
            *v = vec3(
                -1.0 + 2.0 * rng.next_f32(),
                -1.0 + 2.0 * rng.next_f32(),
                -1.0 + 2.0 * rng.next_f32(),
            )
            .normalize();
        }
        randvec
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
            randvec: Perlin::generate(rng),
            perm_x: Perlin::generate_perm(rng),
            perm_y: Perlin::generate_perm(rng),
            perm_z: Perlin::generate_perm(rng),
        }
    }

    #[inline]
    fn interpolate(c: &[[[Vec3; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        for i in 0..2 {
            let ii = i as f32;
            for j in 0..2 {
                let jj = j as f32;
                for k in 0..2 {
                    let kk = k as f32;
                    let weight = vec3(u - ii, v - jj, w - kk);
                    accum += (ii * uu + (1.0 - ii) * (1.0 - uu))
                        * (jj * vv + (1.0 - jj) * (1.0 - vv))
                        * (kk * ww + (1.0 - kk) * (1.0 - ww))
                        * c[i][j][k].dot(weight);
                }
            }
        }
        accum
    }

    pub fn turb(&self, p: Vec3) -> f32 {
        const DEPTH: usize = 7;
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;
        for _ in 0..DEPTH {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }
        accum.abs()
    }

    pub fn noise(&self, p: Vec3) -> f32 {
        let x = p.get_x();
        let y = p.get_y();
        let z = p.get_z();
        let u = x - x.floor();
        let v = y - y.floor();
        let w = z - z.floor();
        let i = x.floor() as usize;
        let j = y.floor() as usize;
        let k = z.floor() as usize;
        let mut c = [[[Vec3::zero(); 2]; 2]; 2];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.randvec[(self.perm_x[(i + di) & 255]
                        ^ self.perm_y[(j + dj) & 255]
                        ^ self.perm_z[(k + dk) & 255])
                        as usize]
                }
            }
        }
        Perlin::interpolate(&c, u, v, w)
    }
}
