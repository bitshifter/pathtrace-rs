#![allow(dead_code)]
use crate::vmath::{vec3, Vec3};

#[derive(Copy, Clone, Debug)]
pub enum Texture<'a> {
    Constant { color: Vec3 },
    Checker { odd: &'a Texture<'a>, even: &'a Texture<'a> },
}

impl<'a> Texture<'a> {
    pub fn value(&self, u: f32, v: f32, p: Vec3) -> Vec3 {
        match self {
            Texture::Constant { color } => {
                *color
            }
            Texture::Checker { odd, even } => {
                // TODO: not implemented
                let s = vec3(10.0, 10.0, 10.0) * p;
                let sines = s.get_x().sin() * s.get_y().sin() * s.get_z().sin();
                if sines < 0.0 {
                    odd.value(u, v, p)
                } else {
                    even.value(u, v, p)
                }
            }
        }
    }
}
