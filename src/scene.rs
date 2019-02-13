use crate::camera::Camera;
use crate::collision::{ray, Ray, RayHit, Sphere, SpheresSoA};
use crate::material::Material;
use crate::math::maxf;
use crate::simd::sinf_cosf;
use glam::{vec3, Vec3};
use rand::{weak_rng, Rng, SeedableRng, XorShiftRng};
use rayon::prelude::*;
use std::f32;
use std::sync::atomic::{AtomicUsize, Ordering};

const MAX_T: f32 = f32::MAX;
const MIN_T: f32 = 0.001;

#[derive(Copy, Clone)]
pub struct Params {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_depth: u32,
    pub random_seed: bool,
}

pub struct Scene {
    spheres: SpheresSoA,
    materials: Vec<Material>,
    emissive: Vec<u32>,
    ray_count: AtomicUsize,
}

impl Scene {
    pub fn new(sphere_materials: &[(Sphere, Material)]) -> Scene {
        let (spheres, materials): (Vec<Sphere>, Vec<Material>) =
            sphere_materials.iter().cloned().unzip();
        let mut emissive = vec![];
        for (index, material) in materials.iter().enumerate() {
            if material.emissive.length_squared() > 0.0 {
                emissive.push(index as u32);
            }
        }
        Scene {
            spheres: SpheresSoA::new(&spheres),
            materials,
            emissive,
            ray_count: AtomicUsize::new(0),
        }
    }

    fn sample_lights(
        &self,
        ray_in: &Ray,
        ray_in_hit: &RayHit,
        in_hit_index: u32,
        attenuation: Vec3,
        rng: &mut XorShiftRng,
        ray_count: &mut usize,
    ) -> Vec3 {
        let mut emissive_out = Vec3::zero();
        for index in &self.emissive {
            if *index == in_hit_index {
                // skip self
                continue;
            }

            // create a random direction towards sphere
            // coord system for sampling: sw, su, sv
            let sphere_centre = self.spheres.centre(*index);
            let sphere_radius_sq = self.spheres.radius_sq(*index);
            let sw = (sphere_centre - ray_in_hit.point).normalize();
            let su = (if sw.get_x().abs() > 0.01 {
                vec3(0.0, 1.0, 0.0)
            } else {
                vec3(1.0, 0.0, 0.0)
            })
            .cross(sw)
            .normalize();
            let sv = sw.cross(su);
            // sample sphere by solid angle
            let cos_a_max = (1.0
                - sphere_radius_sq / (ray_in_hit.point - sphere_centre).length_squared())
            .sqrt();
            let eps1 = rng.next_f32();
            let eps2 = rng.next_f32();
            let cos_a = 1.0 - eps1 + eps1 * cos_a_max;
            let sin_a = (1.0 - cos_a * cos_a).sqrt();
            let phi = 2.0 * f32::consts::PI * eps2;
            let (sin_phi, cos_phi) = sinf_cosf(phi);
            let l = su * (cos_phi * sin_a) + sv * (sin_phi * sin_a) + sw * cos_a;
            //l = normalize(l); // NOTE(fg): This is already normalized, by construction.

            *ray_count += 1;
            let ray_out = ray(ray_in_hit.point, l);
            if let Some((_, out_hit_index)) = self.spheres.ray_hit(&ray_out, MIN_T, MAX_T) {
                if *index == out_hit_index {
                    let omega = 2.0 * f32::consts::PI * (1.0 - cos_a_max);
                    let rdir = ray_in.direction;
                    let nl = if ray_in_hit.normal.dot(rdir) < 0.0 {
                        ray_in_hit.normal
                    } else {
                        -ray_in_hit.normal
                    };
                    let light_emission = self.materials[*index as usize].emissive;
                    emissive_out += (attenuation * light_emission)
                        * (maxf(0.0, l.dot(nl)) * omega / f32::consts::PI);
                }
            }
        }
        emissive_out
    }

    fn ray_trace(
        &self,
        ray_in: &Ray,
        depth: u32,
        max_depth: u32,
        do_material_emission: bool,
        rng: &mut XorShiftRng,
        ray_count: &mut usize,
    ) -> Vec3 {
        *ray_count += 1;
        if let Some((ray_hit, hit_index)) = self.spheres.ray_hit(ray_in, MIN_T, MAX_T) {
            let material = &self.materials[hit_index as usize];
            if depth < max_depth {
                if let Some((attenuation, scattered, do_light_sampling)) =
                    material.scatter(ray_in, &ray_hit, rng)
                {
                    let light_emission = if do_light_sampling {
                        self.sample_lights(ray_in, &ray_hit, hit_index, attenuation, rng, ray_count)
                    } else {
                        Vec3::zero()
                    };
                    // don't do material emission if a previous call has already done explicit
                    // light sampling and added the contribution
                    let material_emission = if do_material_emission {
                        material.emissive
                    } else {
                        Vec3::zero()
                    };
                    let do_material_emission = !do_light_sampling;
                    return material_emission
                        + light_emission
                        + attenuation
                            * self.ray_trace(
                                &scattered,
                                depth + 1,
                                max_depth,
                                do_material_emission,
                                rng,
                                ray_count,
                            );
                }
            }
            return material.emissive;
        } else {
            // sky
            let t = 0.5 * (ray_in.direction.get_y() + 1.0);
            (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0) * 0.3
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
                        col += self.ray_trace(
                            &ray,
                            0,
                            params.max_depth,
                            true,
                            &mut rng,
                            &mut ray_count,
                        );
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
