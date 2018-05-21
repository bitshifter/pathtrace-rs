use camera::Camera;
use glium::{
    self, glutin::{Api, GlProfile, GlRequest}, index::{NoIndices, PrimitiveType},
    texture::buffer_texture::{BufferTexture, BufferTextureType}, vertex::EmptyVertexAttributes,
    Surface,
};
use image;
use scene::{Params, Scene};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::thread;
use std::time::{Duration, SystemTime};

pub fn start_loop(params: Params, camera: Camera, scene: Scene, max_frames: Option<u32>) {
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
    {
        // init buffer texture to something
        let mut mapping = buffer_texture.map();
        for texel in mapping.iter_mut() {
            *texel = (0, 0, 0, 255);
        }
    }

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

    let mut rgb_buffer = Some(vec![
        (0.0, 0.0, 0.0);
        (params.width * params.height) as usize
    ]);

    let (main_send, worker_recv) = channel::<Option<Vec<(f32, f32, f32)>>>();
    let (worker_send, main_recv) = channel::<Vec<(f32, f32, f32)>>();

    thread::spawn(move || {
        let mut frame_num = 0;
        let mut elapsed_secs = 0.0;
        let mut ray_count = 0;
        loop {
            let rgb_buffer = worker_recv.recv().unwrap();
            if let Some(mut rgb_buffer) = rgb_buffer {
                let start_time = SystemTime::now();
                ray_count += scene.update(&params, &camera, frame_num, &mut rgb_buffer);
                frame_num += 1;

                let elapsed = start_time
                    .elapsed()
                    .expect("SystemTime elapsed time failed");
                elapsed_secs +=
                    elapsed.as_secs() as f64 + f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0;

                const RATE: u32 = 10;
                if frame_num % RATE == 0 {
                    let million_ray_count = ray_count as f64 / 1_000_000.0;

                    println!(
                        "{:.2}secs {:.2}Mrays/s {:.2}Mrays/frame {} frames",
                        elapsed_secs / RATE as f64,
                        million_ray_count / elapsed_secs,
                        million_ray_count / RATE as f64,
                        frame_num
                    );

                    elapsed_secs = 0.0;
                    ray_count = 0;
                }

                worker_send.send(rgb_buffer).unwrap();
            } else {
                break;
            }
        }
    });

    let mut frame_num = 0;
    let mut quit = false;
    while !quit {
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
                    }
                    _ => (),
                };
            }
        });

        if quit {
            break;
        }

        // if we own the buffer then send it back to the worker thread
        if let Some(rgb_buffer) = rgb_buffer {
            // send data to worker thread
            main_send.send(Some(rgb_buffer)).unwrap();
        }

        // poll the worker thread to see if it's done
        rgb_buffer = match main_recv.recv_timeout(Duration::from_millis(100)) {
            Ok(rgb_buffer) => {
                // data received - copy to buffer texture
                {
                    let mut mapping = buffer_texture.map();
                    for (texel, rgb) in mapping.iter_mut().zip(rgb_buffer.iter()) {
                        *texel = (
                            (255.99 * rgb.0) as u8,
                            (255.99 * rgb.1) as u8,
                            (255.99 * rgb.2) as u8,
                            255,
                        );
                    }
                }

                // only draw the buffer if we just recieved it
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

                frame_num += 1;
                if let Some(max_frames) = max_frames {
                    if frame_num >= max_frames {
                        quit = true;
                    }
                }

                Some(rgb_buffer)
            }
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => break,
        };
    }

    // reading the front rgb_buffer into an image
    let image: glium::texture::RawImage2d<u8> = display.read_front_buffer();
    let image =
        image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image).flipv().to_rgb();
    image
        .save("output.png")
        .expect("Failed to save output image");

    // tell the worker to exit
    main_send.send(None).unwrap();
}
