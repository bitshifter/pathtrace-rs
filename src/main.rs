#[macro_use]
extern crate clap;
extern crate image;
extern crate rand;
extern crate rayon;

mod camera;
mod math;
mod presets;
mod scene;
mod vmath;

use clap::{App, Arg};
use rand::{Rng, SeedableRng, XorShiftRng};
use rayon::prelude::*;
use std::f32;
use std::time::SystemTime;
use vmath::vec3;

const FIXED_SEED: [u32; 4] = [0x193a_6754, 0xa8a7_d469, 0x9783_0e05, 0x113b_a7bb];

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
            Arg::with_name("preset")
                .help("Scene preset to render")
                .short("P")
                .long("preset")
                .takes_value(true),
        ])
        .get_matches();

    let nx = value_t!(matches, "width", u32).unwrap_or(1280);
    let ny = value_t!(matches, "height", u32).unwrap_or(720);
    let ns = value_t!(matches, "samples", u32).unwrap_or(4);
    let max_depth = value_t!(matches, "depth", u32).unwrap_or(50);
    let preset = matches.value_of("preset").unwrap_or("random");
    let channels = 3;

    println!(
        "generating '{}' preset at {}x{} with {} samples per pixel",
        preset, nx, ny, ns
    );

    let random_seed = matches.is_present("random");
    let weak_rng = || {
        if random_seed {
            rand::weak_rng()
        } else {
            XorShiftRng::from_seed(FIXED_SEED)
        }
    };

    let (scene, camera) = presets::from_name(
        preset,
        &presets::Params {
            width: nx,
            height: ny,
            max_depth,
        },
        &mut weak_rng(),
    ).expect("unrecognised preset");

    let inv_nx = 1.0 / nx as f32;
    let inv_ny = 1.0 / ny as f32;
    let inv_ns = 1.0 / ns as f32;

    let mut buffer = vec![0u8; (nx * ny * channels) as usize];

    let start_time = SystemTime::now();

    scene.update_frame(nx, ny, ns, &mut buffer);

    let elapsed = start_time
        .elapsed()
        .expect("SystemTime elapsed time failed");
    let elapsed_secs = elapsed.as_secs() as f64 + (elapsed.subsec_nanos() as f64) / 1000_000_000.0;
    let ray_count = scene.ray_count();

    println!(
        "{:.2}secs {}rays {:.2}Mrays/s",
        elapsed_secs,
        ray_count,
        ray_count as f64 / 1_000_000.0 / elapsed_secs
    );

    image::save_buffer("output.png", &buffer, nx, ny, image::RGB(8))
        .expect("Failed to save output image");
}
