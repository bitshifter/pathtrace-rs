// TODO: remove
#![allow(unused_imports)]
use crate::{
    camera::Camera,
    collision::{BVHNode, Hitable, HitableList, Ray, Sphere, SpheresSoA, XYRect},
    material::Material,
    perlin::Perlin,
    texture::{RgbImage, Texture},
};
use glam::{vec3, Vec3};
use rand::{weak_rng, Rng, SeedableRng, XorShiftRng};
use rayon::prelude::*;
use std::{
    f32,
    sync::atomic::{AtomicUsize, Ordering},
};
use typed_arena::Arena;

const MAX_T: f32 = f32::MAX;
const MIN_T: f32 = 0.001;

pub struct Storage<'a> {
    pub texture_arena: Arena<Texture<'a>>,
    pub material_arena: Arena<Material<'a>>,
    pub image_arena: Arena<RgbImage>,
    pub sphere_arena: Arena<Sphere>,
    pub xyrect_arena: Arena<XYRect>,
    pub bvhnode_arena: Arena<BVHNode<'a>>,
    pub hitables_arena: Arena<HitableList<'a>>,
    pub perlin_noise: Perlin,
}

impl<'a> Storage<'a> {
    pub fn new(rng: &mut XorShiftRng) -> Storage<'a> {
        Storage {
            texture_arena: Arena::new(),
            material_arena: Arena::new(),
            image_arena: Arena::new(),
            sphere_arena: Arena::new(),
            xyrect_arena: Arena::new(),
            bvhnode_arena: Arena::new(),
            hitables_arena: Arena::new(),
            perlin_noise: Perlin::new(rng),
        }
    }

    #[inline]
    pub fn alloc_texture(&self, texture: Texture<'a>) -> &mut Texture<'a> {
        self.texture_arena.alloc(texture)
    }

    #[inline]
    pub fn alloc_material(&self, material: Material<'a>) -> &mut Material<'a> {
        self.material_arena.alloc(material)
    }

    #[inline]
    pub fn alloc_image(&self, rgb_image: RgbImage) -> &mut RgbImage {
        self.image_arena.alloc(rgb_image)
    }

    #[inline]
    pub fn alloc_sphere(&self, sphere: Sphere) -> &mut Sphere {
        self.sphere_arena.alloc(sphere)
    }

    #[inline]
    pub fn alloc_xyrect(&self, rect: XYRect) -> &mut XYRect {
        self.xyrect_arena.alloc(rect)
    }

    #[inline]
    pub fn alloc_bvhnode(&self, node: BVHNode<'a>) -> &mut BVHNode<'a> {
        self.bvhnode_arena.alloc(node)
    }

    #[inline]
    pub fn alloc_hitables(&self, hitables: Vec<Hitable<'a>>) -> &mut HitableList<'a> {
        self.hitables_arena.alloc(HitableList::new(hitables))
    }
}

#[derive(Copy, Clone)]
pub struct Params {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_depth: u32,
    pub random_seed: bool,
}

pub struct Scene<'a> {
    world: Hitable<'a>,
    ray_count: AtomicUsize,
}

impl<'a> Scene<'a> {
    pub fn new(world: Hitable<'a>) -> Scene<'a> {
        Scene {
            world,
            ray_count: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn sky(ray: &Ray) -> Vec3 {
        // sky
        let t = 0.5 * (ray.direction.get_y() + 1.0);
        (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0) * 0.3
    }

    fn ray_trace_n(
        &self,
        ray_in: &Ray,
        depth: u32,
        max_depth: u32,
        rng: &mut XorShiftRng,
        ray_count: &mut usize,
    ) -> Vec3 {
        *ray_count += 1;
        if let Some((ray_hit, material)) = self.world.ray_hit(ray_in, MIN_T, MAX_T) {
            let emitted = material.emitted(ray_hit.u, ray_hit.v, ray_hit.point);
            if depth < max_depth {
                if let Some((attenuation, scattered)) = material.scatter(ray_in, &ray_hit, rng) {
                    return emitted
                        + attenuation
                            * self.ray_trace_n(&scattered, depth + 1, max_depth, rng, ray_count);
                }
            }
            emitted
        } else {
            Scene::sky(ray_in)
        }
    }

    fn ray_trace_0(
        &self,
        ray_in: &Ray,
        max_depth: u32,
        rng: &mut XorShiftRng,
        ray_count: &mut usize,
    ) -> Vec3 {
        *ray_count += 1;
        // if self.world.in_bounds(&ray_in, MIN_T, MAX_T) {
            if let Some((ray_hit, material)) = self.world.ray_hit(ray_in, MIN_T, MAX_T) {
                let emitted = material.emitted(ray_hit.u, ray_hit.v, ray_hit.point);
                if let Some((attenuation, scattered)) = material.scatter(ray_in, &ray_hit, rng) {
                    return emitted
                        + attenuation * self.ray_trace_n(&scattered, 1, max_depth, rng, ray_count);
                }
                return emitted;
            }
        // }
        Scene::sky(ray_in)
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
                    weak_rng()
                } else {
                    let state = (j as u32 * 9781 + frame_num * 6271) | 1;
                    XorShiftRng::from_seed([state, state, state, state])
                };
                row.iter_mut().enumerate().for_each(|(i, color_out)| {
                    let mut col = Vec3::zero();
                    for _ in 0..params.samples {
                        let u = (i as f32 + rng.next_f32()) * inv_nx;
                        let v = (j as f32 + rng.next_f32()) * inv_ny;
                        let ray = camera.get_ray(u, v, &mut rng);
                        col += self.ray_trace_0(&ray, params.max_depth, &mut rng, &mut ray_count);
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

#[cfg(all(feature = "bench", test))]
mod bench {
    use presets;
    use rand::{SeedableRng, XorShiftRng};
    use scene::{Params, MAX_T, MIN_T};
    use simd::TargetFeature;
    use test::{black_box, Bencher};

    const FIXED_SEED: [u32; 4] = [0x193a_6754, 0xa8a7_d469, 0x9783_0e05, 0x113b_a7bb];
    const PARAMS: Params = Params {
        width: 200,
        height: 100,
        samples: 10,
        max_depth: 10,
        random_seed: false,
    };

    #[bench]
    fn ray_hit_scalar(b: &mut Bencher) {
        let seed = black_box(FIXED_SEED);
        let mut rng = XorShiftRng::from_seed(seed);
        let (scene, camera) = presets::aras_p(&PARAMS);
        let ray = camera.get_ray(0.5, 0.5, &mut rng);
        b.iter(|| scene.spheres.hit_scalar(&ray, MIN_T, MAX_T));
    }

    #[bench]
    fn ray_hit_sse4_1(b: &mut Bencher) {
        let seed = black_box(FIXED_SEED);
        let mut rng = XorShiftRng::from_seed(seed);
        let (scene, camera) = presets::aras_p(&PARAMS);
        let ray = camera.get_ray(0.5, 0.5, &mut rng);
        if scene.feature != TargetFeature::FallBack {
            b.iter(|| unsafe { scene.spheres.hit_sse4_1(&ray, MIN_T, MAX_T) });
        }
    }

    #[bench]
    fn ray_hit_avx2(b: &mut Bencher) {
        let seed = black_box(FIXED_SEED);
        let mut rng = XorShiftRng::from_seed(seed);
        let (scene, camera) = presets::aras_p(&PARAMS);
        let ray = camera.get_ray(0.5, 0.5, &mut rng);
        if scene.feature == TargetFeature::AVX2 {
            b.iter(|| unsafe { scene.spheres.hit_avx2(&ray, MIN_T, MAX_T) });
        }
    }
}
