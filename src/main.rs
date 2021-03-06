#![cfg_attr(feature = "core_intrinsics", feature(core_intrinsics))] // for cttz
#![cfg_attr(feature = "bench", feature(test))] // for bench

#[cfg(feature = "bench")]
extern crate test;

#[cfg(feature = "bench")]
mod bench;
mod camera;
mod collision;
mod glium_window;
mod material;
mod math;
mod offline;
mod params;
mod perlin;
mod presets;
mod scene;
mod simd;
mod storage;
mod texture;

use clap::{value_t, App, Arg};

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
            Arg::with_name("frames")
                .help("Process a fixed number of frames and exit")
                .short("F")
                .long("frames")
                .takes_value(true),
            Arg::with_name("bvh")
                .help("Use bounding volume hierarchy instead of a flat list")
                .short("B")
                .long("bvh"),
            Arg::with_name("offline")
                .help("Don't create a preview render window")
                .short("O")
                .long("offline"),
            Arg::with_name("print")
                .help("Debug print a ray trace and exit")
                .short("X")
                .long("print"),
        ])
        .get_matches();

    let params = params::Params {
        width: value_t!(matches, "width", u32).unwrap_or(1280),
        height: value_t!(matches, "height", u32).unwrap_or(720),
        samples: value_t!(matches, "samples", u32).unwrap_or(4),
        max_depth: value_t!(matches, "depth", u32).unwrap_or(10),
        random_seed: matches.is_present("random"),
        use_bvh: matches.is_present("bvh"),
    };

    let preset = matches.value_of("preset").unwrap_or("two_perlin_spheres");

    if matches.is_present("print") {
        offline::print_ray_trace(preset, params);
    } else if matches.is_present("offline") {
        offline::render_offline(preset, params);
    } else {
        let max_frames = value_t!(matches, "frames", u32).ok().and_then(Some);
        glium_window::start_loop(preset, params, max_frames);
    }
}
