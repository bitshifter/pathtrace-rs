// The tracer lives in the library target (`src/lib.rs`) so that the criterion benchmarks in
// `benches/` can build scenes and rays through the same API as the tracer itself.
use pathtrace_rs::{offline, params, pixels_window, presets};

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
                .default_value("random_spheres")
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
            Arg::new("soa")
                .help("Sphere structure of arrays collision: auto uses SoA for all-sphere scenes")
                .short('A')
                .long("soa")
                .default_value("auto")
                .value_parser(PossibleValuesParser::new(["auto", "on", "off"])),
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
        soa: match matches.get_one::<String>("soa").map(String::as_str) {
            Some("on") => params::SoaMode::On,
            Some("off") => params::SoaMode::Off,
            _ => params::SoaMode::Auto,
        },
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
