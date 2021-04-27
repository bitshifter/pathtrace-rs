#![allow(dead_code)]
use crate::perlin::Perlin;
use glam::{vec3, Vec3};

#[derive(Clone, Debug)]
pub struct RgbImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl RgbImage {
    // TODO: error handling
    pub fn open(path: &str) -> RgbImage {
        let image = image::open(path).unwrap();
        let image = image.to_rgb8();
        let width = image.width();
        let height = image.height();
        let data = image.into_raw();
        RgbImage {
            data,
            width,
            height,
        }
    }

    pub fn value(&self, u: f32, v: f32) -> Vec3 {
        let i = (u * self.width as f32) as i32;
        let j = ((1.0 - v) * self.height as f32 - 0.001) as i32;
        let i = i.max(0).min(self.width as i32 - 1) as usize;
        let j = j.max(0).min(self.height as i32 - 1) as usize;
        let r = self.data[3 * i + 3 * self.width as usize * j] as f32 / 255.0;
        let g = self.data[3 * i + 3 * self.width as usize * j + 1] as f32 / 255.0;
        let b = self.data[3 * i + 3 * self.width as usize * j + 2] as f32 / 255.0;
        vec3(r, g, b)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Texture<'a> {
    Image {
        image: &'a RgbImage,
    },
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

pub fn rgb_image<'a>(image: &'a RgbImage) -> Texture<'a> {
    Texture::Image { image }
}

impl<'a> Texture<'a> {
    pub fn value(&self, u: f32, v: f32, p: Vec3) -> Vec3 {
        match self {
            Texture::Image { image } => image.value(u, v),
            Texture::Constant { color } => *color,
            Texture::Checker { odd, even } => {
                let s = vec3(10.0, 10.0, 10.0) * p;
                let sines = s.x.sin() * s.y.sin() * s.z.sin();
                if sines < 0.0 {
                    odd.value(u, v, p)
                } else {
                    even.value(u, v, p)
                }
            }
            Texture::Noise { noise, scale } => {
                vec3(1.0, 1.0, 1.0) * 0.5 * (1.0 + (scale * p.z + 10.0 * noise.turb(p)).sin())
            }
        }
    }
}
