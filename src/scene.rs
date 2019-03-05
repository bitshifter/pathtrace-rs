use crate::{
    camera::Camera,
    collision::{BVHNode, Hitable, HitableList, MovingSphere, Ray, Rect, Sphere},
    material::Material,
    perlin::Perlin,
    texture::{RgbImage, Texture},
};
use glam::{vec3, Vec3};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256Plus;
use rayon::prelude::*;
use std::{
    f32,
    sync::atomic::{AtomicUsize, Ordering},
};
use typed_arena::Arena;

pub const MAX_T: f32 = f32::MAX;
pub const MIN_T: f32 = 0.001;

pub struct Storage<'a> {
    pub texture_arena: Arena<Texture<'a>>,
    pub material_arena: Arena<Material<'a>>,
    pub image_arena: Arena<RgbImage>,
    pub sphere_arena: Arena<Sphere>,
    pub moving_sphere_arena: Arena<MovingSphere>,
    pub rect_arena: Arena<Rect>,
    pub bvhnode_arena: Arena<BVHNode<'a>>,
    pub hitables_arena: Arena<HitableList<'a>>,
    pub perlin_noise: Perlin,
}

impl<'a> Storage<'a> {
    pub fn new(rng: &mut Xoshiro256Plus) -> Storage<'a> {
        Storage {
            texture_arena: Arena::new(),
            material_arena: Arena::new(),
            image_arena: Arena::new(),
            moving_sphere_arena: Arena::new(),
            sphere_arena: Arena::new(),
            rect_arena: Arena::new(),
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
    pub fn alloc_moving_sphere(&self, sphere: MovingSphere) -> &mut MovingSphere {
        self.moving_sphere_arena.alloc(sphere)
    }

    #[inline]
    pub fn alloc_rect(&self, rect: Rect) -> &mut Rect {
        self.rect_arena.alloc(rect)
    }

    #[inline]
    pub fn alloc_hitables(&self, hitables: Vec<Hitable<'a>>) -> &mut HitableList<'a> {
        self.hitables_arena.alloc(HitableList::new(hitables))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Params {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_depth: u32,
    pub random_seed: bool,
    pub use_bvh: bool,
}

impl Params {
    pub fn new_rng(&self) -> Xoshiro256Plus {
        if self.random_seed {
            Xoshiro256Plus::seed_from_u64(rand::random())
        } else {
            Xoshiro256Plus::seed_from_u64(0)
        }
    }

    pub fn new_scene<'a>(
        &self,
        rng: &mut Xoshiro256Plus,
        storage: &'a Storage<'a>,
        mut hitables: Vec<Hitable<'a>>,
        sky: Option<Vec3>,
    ) -> Scene<'a> {
        let hitable_list = if self.use_bvh {
            let bvh_root = BVHNode::new(rng, &mut hitables, &storage.bvhnode_arena).unwrap();
            dbg!(bvh_root.get_stats());

            Hitable::BVHNode(bvh_root)
        } else {
            Hitable::List(storage.alloc_hitables(hitables))
        };

        Scene::new(hitable_list, sky)
    }
}

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
