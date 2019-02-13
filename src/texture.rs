#![allow(dead_code)]
use crate::{
    perlin::Perlin,
    vmath::{vec3, Vec3},
};

#[derive(Copy, Clone, Debug)]
pub enum Texture<'a> {
    Constant {
        color: Vec3,
    },
    Checker {
        odd: &'a Texture<'a>,
        even: &'a Texture<'a>,
    },
    Noise {
        noise: &'a Perlin,
        scale: f32,
    },
}

pub fn constant<'a>(color: Vec3) -> Texture<'a> {
    Texture::Constant { color }
}

pub fn checker<'a>(odd: &'a Texture<'a>, even: &'a Texture<'a>) -> Texture<'a> {
    Texture::Checker { odd, even }
}

pub fn noise<'a>(noise: &'a Perlin, scale: f32) -> Texture<'a> {
    Texture::Noise { noise, scale }
}

impl<'a> Texture<'a> {
    pub fn value(&self, u: f32, v: f32, p: Vec3) -> Vec3 {
        match self {
            Texture::Constant { color } => *color,
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
            Texture::Noise { noise, scale } => {
                vec3(1.0, 1.0, 1.0) * 0.5 * (1.0 + (scale * p.get_z() + 10.0 * noise.turb(p)).sin())
            }
        }
    }
}
