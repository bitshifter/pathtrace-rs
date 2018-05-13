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
use glium::{Surface, glutin::{Api, GlProfile, GlRequest}, index::{NoIndices, PrimitiveType},
            texture::buffer_texture::{BufferTexture, BufferTextureType},
            vertex::EmptyVertexAttributes};
use std::time::SystemTime;

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

    let params = scene::Params {
        width: value_t!(matches, "width", u32).unwrap_or(1280),
        height: value_t!(matches, "height", u32).unwrap_or(720),
        samples: value_t!(matches, "samples", u32).unwrap_or(4),
        max_depth: value_t!(matches, "depth", u32).unwrap_or(50),
        random_seed: matches.is_present("random"),
    };

    let preset = matches.value_of("preset").unwrap_or("aras");

    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(params.width, params.height)
        .with_title("pathtrace-rs");
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 2)))
        .with_gl_profile(GlProfile::Core);
    let display =
        glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    let mut buffer_texture: BufferTexture<(u8, u8, u8, u8)> =
        BufferTexture::empty_persistent(
            &display,
            (params.width * params.height * 4) as usize,
            BufferTextureType::Float,
        ).expect("Failed to create rgb_buffer texture");

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
        preset, params.width, params.height, params.samples
    );

    let (scene, camera) = presets::from_name(preset, &params).expect("unrecognised preset");

    let mut rgb_buffer = vec![(0.0, 0.0, 0.0); (params.width * params.height) as usize];

    let mut frame_num = 0;
    loop {
        let mut quit = false;
        events_loop.poll_events(|event| {
            use glium::glutin::{ElementState, Event, VirtualKeyCode, WindowEvent};
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::Closed => quit = true,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let ElementState::Released = input.state {
                            if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                                quit = true;
                            }
                        }
                    },
                    _ => (),
                };
            }
        });
        if quit {
            break;
        }

        let start_time = SystemTime::now();
        let ray_count = scene.update(&params, &camera, frame_num, &mut rgb_buffer);
        frame_num += 1;

        let elapsed = start_time
            .elapsed()
            .expect("SystemTime elapsed time failed");
        let elapsed_secs =
            elapsed.as_secs() as f64 + (elapsed.subsec_nanos() as f64) / 1_000_000_000.0;
        let ray_count = ray_count as f64 / 1_000_000.0;

        println!(
            "{:.2}secs {:.2}Mrays/s {:.2}Mrays/frame {}frames",
            elapsed_secs,
            ray_count / elapsed_secs,
            ray_count,
            frame_num
        );

        {
            let mut mapping = buffer_texture.map();
            for (color_out, color_in) in mapping.iter_mut().zip(rgb_buffer.iter()) {
                color_out.0 = (255.99 * color_in.0) as u8;
                color_out.1 = (255.99 * color_in.1) as u8;
                color_out.2 = (255.99 * color_in.2) as u8;
                color_out.3 = 255;
            }
        }

        let mut target = display.draw();
        target
            .draw(
                EmptyVertexAttributes { len: 4 },
                NoIndices(PrimitiveType::TriangleStrip),
                &program,
                &uniform!{ tex: &buffer_texture, stride: params.width as i32 },
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();
    }

    // reading the front rgb_buffer into an image
    let image: glium::texture::RawImage2d<u8> = display.read_front_buffer();
    let image =
        image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image).flipv().to_rgb();
    image
        .save("output.png")
        .expect("Failed to save output image");
}
