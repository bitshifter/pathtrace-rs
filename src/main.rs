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
use rand::{thread_rng, Rng, SeedableRng, XorShiftRng};
use rayon::prelude::*;
use scene::Scene;
use std::f32;
use std::time::SystemTime;
use vmath::vec3;

const FIXED_SEED: [u32; 4] = [0x193a6754, 0xa8a7d469, 0x97830e05, 0x113ba7bb];

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

    let seed = if matches.is_present("random") {
        thread_rng().gen()
    } else {
        FIXED_SEED
    };

    let mut rng = XorShiftRng::from_seed(seed);
    let scene = Scene::random_scene(&mut rng);

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

    buffer
        .par_chunks_mut((nx * channels) as usize)
        .rev()
        .enumerate()
        .for_each(|(j, row)| {
            let mut rng = XorShiftRng::from_seed(seed);
            for (i, rgb) in row.chunks_mut(channels as usize).enumerate() {
                let mut col = vec3(0.0, 0.0, 0.0);
                for _ in 0..ns {
                    let u = (i as f32 + rng.next_f32()) / nx as f32;
                    let v = (j as f32 + rng.next_f32()) / ny as f32;
                    let ray = camera.get_ray(u, v, &mut rng);
                    col += scene.ray_trace(&ray, 0, &mut rng);
                }
                col /= ns as f32;
                let mut iter = rgb.iter_mut();
                *iter.next().unwrap() = (255.99 * col.x.sqrt()) as u8;
                *iter.next().unwrap() = (255.99 * col.y.sqrt()) as u8;
                *iter.next().unwrap() = (255.99 * col.z.sqrt()) as u8;
            }
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
