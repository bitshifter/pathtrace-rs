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
use rand::Rng;
use rayon::prelude::*;
use scene::Scene;
use std::f32;
use std::time::SystemTime;
use vmath::vec3;

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

    // Not writing crypto so use a weak rng, also pass it around to avoid construction cost.
    let scene = Scene::random_scene(&mut rand::weak_rng());

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

    let mut buffer: Vec<u8> = std::iter::repeat(0)
        .take((nx * ny * channels) as usize)
        .collect();

    let start_time = SystemTime::now();

    // parallel iterate each row of pixels
    buffer
        .par_chunks_mut((nx * channels) as usize)
        .rev()
        .enumerate()
        .for_each(|(j, row)| {
            for (i, rgb) in row.chunks_mut(channels as usize).enumerate() {
                let mut rng = rand::weak_rng();
                let mut col = vec3(0.0, 0.0, 0.0);
                for _ in 0..ns {
                    let u = (i as f32 + rng.next_f32()) * inv_nx;
                    let v = (j as f32 + rng.next_f32()) * inv_ny;
                    let ray = camera.get_ray(u, v, &mut rng);
                    col += scene.ray_trace(&ray, 0, &mut rng);
                }
                col *= inv_ns;
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
