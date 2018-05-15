#[macro_use]
extern crate clap;
#[macro_use]
extern crate glium;
extern crate image;
extern crate rand;
extern crate rayon;

mod camera;
mod math;
mod presets;
mod scene;
mod vmath;

use clap::{App, Arg};
use glium::{Surface, index::{NoIndices, PrimitiveType},
            texture::buffer_texture::{BufferTexture, BufferTextureType},
            vertex::EmptyVertexAttributes};
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
    let preset = matches.value_of("preset").unwrap_or("aras");
    let channels = 3;

    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(nx, ny)
        .with_title("pathtrace-rs");
    let context = glium::glutin::ContextBuilder::new();
    let display =
        glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let mut buf_tex: BufferTexture<(u8, u8, u8, u8)> =
        BufferTexture::empty_persistent(&display, (nx * ny * 4) as usize, BufferTextureType::Float)
            .expect("Failed to create buffer texture");

    let program = glium::Program::from_source(
        &display,
        "
            #version 330 core

            void main() {
                const vec4 vertices[] = vec4[](vec4(-1.0, -1.0, 0.5, 1.0),
                                               vec4( 1.0, -1.0, 0.5, 1.0),
                                               vec4(-1.0,  1.0, 0.5, 1.0),
                                               vec4( 1.0,  1.0, 0.5, 1.0));

                gl_Position = vertices[gl_VertexID];
            }
        ",
        "
            #version 330 core

            uniform int stride;
            uniform samplerBuffer tex;
            out vec4 color;

            void main() {
                int x = int(gl_FragCoord.x);
                int y = int(gl_FragCoord.y);
                int index = y * stride + x;
                color = texelFetch(tex, index);
            }
        ",
        None,
    ).expect("Failed to create shader");

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

    // parallel iterate each row of pixels
    buffer
        .par_chunks_mut((nx * channels) as usize)
        .enumerate()
        .for_each(|(j, row)| {
            row.chunks_mut(channels as usize)
                .enumerate()
                .for_each(|(i, rgb)| {
                    let mut rng = weak_rng();
                    let mut col = vec3(0.0, 0.0, 0.0);
                    for _ in 0..ns {
                        let u = (i as f32 + rng.next_f32()) * inv_nx;
                        let v = (j as f32 + rng.next_f32()) * inv_ny;
                        let ray = camera.get_ray(u, v, &mut rng);
                        col += scene.ray_trace(&ray, 0, &mut rng);
                    }
                    col *= inv_ns;
                    let mut iter = rgb.iter_mut();
                    *iter.next().unwrap() = (255.99 * col.x) as u8;
                    *iter.next().unwrap() = (255.99 * col.y) as u8;
                    *iter.next().unwrap() = (255.99 * col.z) as u8;
                });
        });

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

    {
        let mut mapping = buf_tex.map();
        for (texel, rgb) in mapping.iter_mut().zip(buffer.chunks(channels as usize)) {
            let mut iter = rgb.iter();
            texel.0 = *iter.next().unwrap();
            texel.1 = *iter.next().unwrap();
            texel.2 = *iter.next().unwrap();
        }
    }

    loop {
        let mut quit = false;
        events_loop.poll_events(|event| {
            use glium::glutin::{ElementState, Event, VirtualKeyCode, WindowEvent};
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::Closed => quit = true,
                    WindowEvent::KeyboardInput { input, .. } => match input.state {
                        ElementState::Released => match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => quit = true,
                            _ => (),
                        },
                        _ => (),
                    },
                    _ => (),
                };
            }
        });
        if quit {
            break;
        }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                EmptyVertexAttributes { len: 4 },
                NoIndices(PrimitiveType::TriangleStrip),
                &program,
                &uniform!{ tex: &buf_tex, stride: nx as i32 },
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    }

    image::save_buffer("output.png", &buffer, nx, ny, image::RGB(8))
        .expect("Failed to save output image");
}
