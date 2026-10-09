#![cfg_attr(feature = "core_intrinsics", feature(core_intrinsics))] // for cttz
#![cfg_attr(feature = "bench", feature(test))] // for bench

#[cfg(feature = "bench")]
extern crate test;

#[cfg(feature = "bench")]
mod bench;
mod camera;
mod collision;
mod material;
mod math;
mod offline;
mod params;
mod perlin;
mod pixels_window;
mod presets;
mod scene;
mod simd;
mod storage;
mod texture;

use clap::{Arg, ArgAction, Command, builder::PossibleValuesParser, value_parser};

fn main() {
    let matches = Command::new("Toy Path Tracer")
        .version("0.1")
        .args([
            Arg::new("width")
                .help("Image width to generate")
                .short('W')
                .long("width")
                .default_value("1280")
                .value_parser(value_parser!(u32)),
            Arg::new("height")
                .help("Image height to generate")
                .short('H')
                .long("height")
                .default_value("720")
                .value_parser(value_parser!(u32)),
            Arg::new("samples")
                .help("Number of samples per pixel")
                .short('S')
                .long("samples")
                .default_value("4")
                .value_parser(value_parser!(u32)),
            Arg::new("depth")
                .help("Maximum bounces per ray")
                .short('D')
                .long("depth")
                .default_value("10")
                .value_parser(value_parser!(u32)),
            Arg::new("random")
                .help("Use a random seed")
                .short('R')
                .long("random")
                .action(ArgAction::SetTrue),
            Arg::new("preset")
                .help("Scene preset to render")
                .short('P')
                .long("preset")
                .default_value("two_perlin_spheres")
                .value_parser(PossibleValuesParser::new(presets::NAMES)),
            Arg::new("frames")
                .help("Process a fixed number of frames and exit")
                .short('F')
                .long("frames")
                .value_parser(value_parser!(u32)),
            Arg::new("bvh")
                .help("Use bounding volume hierarchy instead of a flat list")
                .short('B')
                .long("bvh")
                .action(ArgAction::SetTrue),
            Arg::new("offline")
                .help("Don't create a preview render window")
                .short('O')
                .long("offline")
                .action(ArgAction::SetTrue),
            Arg::new("print")
                .help("Debug print a ray trace and exit")
                .short('X')
                .long("print")
                .action(ArgAction::SetTrue),
        ])
        .get_matches();

    let params = params::Params {
        width: *matches.get_one::<u32>("width").unwrap(),
        height: *matches.get_one::<u32>("height").unwrap(),
        samples: *matches.get_one::<u32>("samples").unwrap(),
        max_depth: *matches.get_one::<u32>("depth").unwrap(),
        random_seed: matches.get_flag("random"),
        use_bvh: matches.get_flag("bvh"),
    };

    let preset = matches.get_one::<String>("preset").unwrap();

    if matches.get_flag("print") {
        offline::print_ray_trace(preset, params);
    } else if matches.get_flag("offline") {
        offline::render_offline(preset, params);
    } else {
        let max_frames = matches.get_one::<u32>("frames").copied();
        pixels_window::start_loop(preset, params, max_frames);
    }
}
