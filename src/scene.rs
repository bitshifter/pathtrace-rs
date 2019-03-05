use crate::{
    camera::Camera,
    collision::{Hitable, Ray},
    params::Params,
};
use glam::{vec3, Vec3};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;
use rayon::prelude::*;
use std::{
    f32,
    sync::atomic::{AtomicUsize, Ordering},
};

pub const MAX_T: f32 = f32::MAX;
pub const MIN_T: f32 = 0.001;

pub struct Scene<'a> {
    world: Hitable<'a>,
    sky: Option<Vec3>,
    ray_count: AtomicUsize,
}

impl<'a> Scene<'a> {
    pub fn new(world: Hitable<'a>, sky: Option<Vec3>) -> Scene<'a> {
        Scene {
            world,
            sky,
            ray_count: AtomicUsize::new(0),
        }
    }

    pub fn print_ray_trace(&self, ray: &Ray) {
        if let Hitable::BVHNode(node) = self.world {
            node.print_ray_hit(ray, MIN_T, MAX_T);
        }
    }

    #[inline]
    fn sky(&self, ray: &Ray) -> Vec3 {
        if let Some(sky) = self.sky {
            sky
        } else {
            let t = 0.5 * (ray.direction.get_y() + 1.0);
            Vec3::splat(1.0 - t) + t * vec3(0.5, 0.7, 1.0) * 0.3
        }
    }

    fn ray_trace(
        &self,
        ray_in: &Ray,
        depth: u32,
        max_depth: u32,
        rng: &mut Xoshiro256Plus,
        ray_count: &mut usize,
    ) -> Vec3 {
        *ray_count += 1;
        if let Some((ray_hit, material)) = self.world.ray_hit(ray_in, MIN_T, MAX_T) {
            let emitted = material.emitted(ray_hit.u, ray_hit.v, ray_hit.point);
            if depth < max_depth {
                if let Some((attenuation, scattered)) = material.scatter(ray_in, &ray_hit, rng) {
                    return emitted
                        + attenuation
                            * self.ray_trace(&scattered, depth + 1, max_depth, rng, ray_count);
                }
            }
            emitted
        } else {
            self.sky(ray_in)
        }
    }

    pub fn update(
        &self,
        params: &Params,
        camera: &Camera,
        frame_num: u32,
        buffer: &mut [(f32, f32, f32)],
    ) -> usize {
        self.ray_count.store(0, Ordering::Relaxed);

        let inv_nx = 1.0 / params.width as f32;
        let inv_ny = 1.0 / params.height as f32;
        let inv_ns = 1.0 / params.samples as f32;

        let mix_prev = frame_num as f32 / (frame_num + 1) as f32;
        let mix_new = 1.0 - mix_prev;

        // parallel iterate each row of pixels
        buffer
            .par_chunks_mut(params.width as usize)
            .enumerate()
            .for_each(|(j, row)| {
                let mut ray_count = 0;
                let mut rng = if params.random_seed {
                    Xoshiro256Plus::seed_from_u64(rand::random())
                } else {
                    Xoshiro256Plus::seed_from_u64((j as u64 * 9781 + frame_num as u64 * 6271) | 1)
                };
                row.iter_mut().enumerate().for_each(|(i, color_out)| {
                    let mut col = Vec3::zero();
                    for _ in 0..params.samples {
                        let u = (i as f32 + rng.gen::<f32>()) * inv_nx;
                        let v = (j as f32 + rng.gen::<f32>()) * inv_ny;
                        let ray = camera.get_ray(u, v, &mut rng);
                        col += self.ray_trace(&ray, 0, params.max_depth, &mut rng, &mut ray_count);
                    }
                    col *= inv_ns;
                    color_out.0 = color_out.0 * mix_prev + col.get_x() * mix_new;
                    color_out.1 = color_out.1 * mix_prev + col.get_y() * mix_new;
                    color_out.2 = color_out.2 * mix_prev + col.get_z() * mix_new;
                });
                self.ray_count.fetch_add(ray_count, Ordering::Relaxed);
            });
        self.ray_count.load(Ordering::Relaxed)
    }
}
