#[macro_use]
extern crate clap;
extern crate image;
extern crate rand;
extern crate rayon;

mod camera;
mod scene;
mod vmath;

use camera::Camera;
use clap::{App, Arg};
use rand::{Rng, FromEntropy, SeedableRng};
use rand::rngs::SmallRng;
use rayon::prelude::*;
use scene::Scene;
use std::f32;
use std::time::SystemTime;
use vmath::vec3;

const FIXED_SEED: [u8; 16] = [0x19, 0x3a, 0x67, 0x54, 0xa8, 0xa7, 0xd4, 0x69, 0x97, 0x83, 0x0, 0xe05, 0x11, 0x3b, 0xa7, 0xbb];

fn main() {
    let matches = App::new("Toy Path Tracer")
        .version("0.1")
        .args(&[
            Arg::with_name("width")
                .help("Image width to generate")
                .short("W")
                .long("width")
                .takes_value(true),
            Arg::with_name("height")
                .help("Image height to generate")
                .short("H")
                .long("height")
                .takes_value(true),
            Arg::with_name("samples")
                .help("Number of samples per pixel")
                .short("S")
                .long("samples")
                .takes_value(true),
            Arg::with_name("depth")
                .help("Maximum bounces per ray")
                .short("D")
                .long("depth")
                .takes_value(true),
            Arg::with_name("random")
                .help("Use a random seed")
                .short("R")
                .long("random"),
        ])
        .get_matches();

    let nx = value_t!(matches, "width", u32).unwrap_or(1200);
    let ny = value_t!(matches, "height", u32).unwrap_or(800);
    let ns = value_t!(matches, "samples", u32).unwrap_or(10);
    let channels = 3;

    println!(
        "generating {}x{} image with {} samples per pixel",
        nx, ny, ns
    );

    let inv_nx = 1.0 / nx as f32;
    let inv_ny = 1.0 / ny as f32;
    let inv_ns = 1.0 / ns as f32;

    let random_seed = matches.is_present("random");
    let weak_rng = || {
        if random_seed {
            SmallRng::from_entropy()
        } else {
            SmallRng::from_seed(FIXED_SEED)
        }
    };

    let max_depth = value_t!(matches, "depth", u32).unwrap_or(50);
    let scene = Scene::random_scene(max_depth, &mut weak_rng());

    let lookfrom = vec3(13.0, 2.0, 3.0);
    let lookat = vec3(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        20.0,
        nx as f32 / ny as f32,
        aperture,
        dist_to_focus,
    );

    let mut buffer = vec![0u8; (nx * ny * channels) as usize];

    let start_time = SystemTime::now();

    // parallel iterate each row of pixels
    buffer
        .par_chunks_mut((nx * channels) as usize)
        .rev()
        .enumerate()
        .for_each(|(j, row)| {
            row.chunks_mut(channels as usize)
                .enumerate()
                .for_each(|(i, rgb)| {
                    let mut rng = weak_rng();
                    let mut col = vec3(0.0, 0.0, 0.0);
                    for _ in 0..ns {
                        let u = (i as f32 + rng.gen::<f32>()) * inv_nx;
                        let v = (j as f32 + rng.gen::<f32>()) * inv_ny;
                        let ray = camera.get_ray(u, v, &mut rng);
                        col += scene.ray_trace(&ray, 0, &mut rng);
                    }
                    col *= inv_ns;
                    let mut iter = rgb.iter_mut();
                    *iter.next().unwrap() = (255.99 * col.x.sqrt()) as u8;
                    *iter.next().unwrap() = (255.99 * col.y.sqrt()) as u8;
                    *iter.next().unwrap() = (255.99 * col.z.sqrt()) as u8;
                });
        });

    let elapsed = start_time
        .elapsed()
        .expect("SystemTime elapsed time failed");

    println!(
        "{}.{:.2} seconds {} rays",
        elapsed.as_secs(),
        elapsed.subsec_nanos(),
        scene.ray_count()
    );

    image::save_buffer("output.png", &buffer, nx, ny, image::RGB(8))
        .expect("Failed to save output image");
}
