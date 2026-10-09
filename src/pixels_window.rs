use crate::{params::Params, presets, storage::Storage};
use pixels::{Pixels, ScalingMode, SurfaceTexture};
use std::{
    sync::{
        Arc,
        mpsc::{Receiver, RecvTimeoutError, Sender, channel},
    },
    thread,
    time::{Duration, SystemTime},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

type RgbBuffer = Vec<(f32, f32, f32)>;

/// Copy an rgb frame buffer into a pixels RGBA frame buffer, converting to u8.
///
/// The scene buffer has row 0 at the bottom (it is written back to front, see
/// `offline::render_offline`), while `pixels` expects row 0 at the top, so the
/// rows are flipped on the way in.
fn blit(frame: &mut [u8], rgb_buffer: &[(f32, f32, f32)], width: usize) {
    for (frame_row, buf_row) in frame
        .as_chunks_mut::<4>()
        .0
        .chunks_mut(width)
        .rev()
        .zip(rgb_buffer.chunks(width))
    {
        for (rgba, rgb) in frame_row.iter_mut().zip(buf_row) {
            rgba.copy_from_slice(&[
                (255.99 * rgb.0.clamp(0.0, 1.0)) as u8,
                (255.99 * rgb.1.clamp(0.0, 1.0)) as u8,
                (255.99 * rgb.2.clamp(0.0, 1.0)) as u8,
                255,
            ]);
        }
    }
}

pub fn start_loop(preset: &str, params: Params, max_frames: Option<u32>) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::new(preset, params, max_frames);
    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
    app.finish();
}

struct App {
    params: Params,
    max_frames: Option<u32>,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    main_send: Sender<Option<RgbBuffer>>,
    main_recv: Receiver<RgbBuffer>,
    frame_num: u32,
    save: bool,
}

impl App {
    fn new(preset: &str, params: Params, max_frames: Option<u32>) -> Self {
        let (main_send, worker_recv) = channel::<Option<RgbBuffer>>();
        let (worker_send, main_recv) = channel::<RgbBuffer>();

        let preset = preset.to_string();
        thread::spawn(move || {
            let mut rng = params.new_rng();

            let storage = Storage::new(&mut rng);
            let (hitables, camera, sky) = presets::from_name(&preset, &params, &mut rng, &storage)
                .expect("unrecognised preset");

            let scene = params.new_scene(&mut rng, &storage, hitables, sky);

            let mut frame_num = 0;
            let mut elapsed_count = 0;
            let mut elapsed_secs = 0.0;
            let mut ray_count = 0;
            loop {
                let rgb_buffer = worker_recv.recv().unwrap();
                if let Some(mut rgb_buffer) = rgb_buffer {
                    let start_time = SystemTime::now();
                    ray_count += scene.update(&params, &camera, frame_num, &mut rgb_buffer);
                    frame_num += 1;
                    elapsed_count += 1;

                    let elapsed = start_time
                        .elapsed()
                        .expect("SystemTime elapsed time failed");
                    elapsed_secs += elapsed.as_secs() as f64
                        + f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0;

                    const RATE: u32 = 10;

                    if elapsed_secs > 10.0 || elapsed_count == RATE {
                        let million_ray_count = ray_count as f64 / 1_000_000.0;

                        println!(
                            "{:.2}secs {:.2}Mrays/s {:.2}Mrays/frame {} frames",
                            elapsed_secs / elapsed_count as f64,
                            million_ray_count / elapsed_secs,
                            million_ray_count / elapsed_count as f64,
                            frame_num
                        );

                        elapsed_secs = 0.0;
                        elapsed_count = 0;
                        ray_count = 0;
                    }

                    worker_send.send(rgb_buffer).unwrap();
                } else {
                    break;
                }
            }
        });

        // hand the initial (black) buffer to the worker
        let initial = vec![(0.0, 0.0, 0.0); (params.width * params.height) as usize];
        main_send.send(Some(initial)).unwrap();

        Self {
            params,
            max_frames,
            window: None,
            pixels: None,
            main_send,
            main_recv,
            frame_num: 0,
            save: false,
        }
    }

    fn finish(&mut self) {
        // tell the worker to exit
        self.main_send.send(None).ok();

        if self.save
            && let Some(pixels) = &self.pixels
        {
            let frame = pixels.frame();
            let image =
                image::RgbaImage::from_raw(self.params.width, self.params.height, frame.to_vec())
                    .expect("Failed to create image buffer");
            // pixels.frame() row 0 is the top row, same as the output image
            image::DynamicImage::ImageRgba8(image)
                .to_rgb8()
                .save("output.png")
                .expect("Failed to save output image");
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.pixels.is_some() {
            return;
        }

        let size = LogicalSize::new(f64::from(self.params.width), f64::from(self.params.height));
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("pathtrace-rs")
                        .with_inner_size(size),
                )
                .expect("Failed to create window"),
        );

        let inner_size = window.inner_size();
        let surface_texture =
            SurfaceTexture::new(inner_size.width, inner_size.height, window.clone());
        let mut pixels = Pixels::new(self.params.width, self.params.height, surface_texture)
            .expect("Failed to create display");
        pixels.set_scaling_mode(ScalingMode::Fill);

        self.pixels = Some(pixels);
        self.window = Some(window);

        // poll the worker thread from `about_to_wait`
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.save = true;
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Released
                    && event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                {
                    self.save = true;
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                if let Some(pixels) = &mut self.pixels
                    && let Err(err) = pixels.resize_surface(size.width, size.height)
                {
                    eprintln!("pixels.resize_surface() failed: {err}");
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels
                    && let Err(err) = pixels.render()
                {
                    eprintln!("pixels.render() failed: {err}");
                    event_loop.exit();
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // poll the worker thread to see if it's done
        let rgb_buffer = match self.main_recv.recv_timeout(Duration::from_millis(100)) {
            Ok(rgb_buffer) => rgb_buffer,
            Err(RecvTimeoutError::Timeout) => return,
            Err(RecvTimeoutError::Disconnected) => {
                event_loop.exit();
                return;
            }
        };

        // data received - copy it into the pixel buffer
        if let Some(pixels) = &mut self.pixels {
            blit(pixels.frame_mut(), &rgb_buffer, self.params.width as usize);
        }

        // give the buffer back to the worker thread for the next sample
        self.main_send.send(Some(rgb_buffer)).unwrap();

        // only draw the buffer if we just received it
        if let Some(window) = &self.window {
            window.request_redraw();
        }

        self.frame_num += 1;
        if let Some(max_frames) = self.max_frames
            && self.frame_num >= max_frames
        {
            event_loop.exit();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::blit;

    #[test]
    fn blit_flips_rows_vertically() {
        // scene buffer row 0 is the bottom row of the image
        let rgb = vec![
            (1.0, 0.0, 0.0),
            (1.0, 0.0, 0.0), // bottom row, red
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 0.0), // top row, green
        ];
        let mut frame = [0u8; 2 * 2 * 4];
        blit(&mut frame, &rgb, 2);

        let row = |i: usize| &frame[i * 2 * 4..(i + 1) * 2 * 4];
        // pixels frame row 0 is the top row
        assert_eq!(row(0), &[0, 255, 0, 255, 0, 255, 0, 255]);
        assert_eq!(row(1), &[255, 0, 0, 255, 255, 0, 0, 255]);
    }
}
