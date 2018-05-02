#[macro_use]
extern crate clap;
extern crate image;
extern crate rand;

mod camera;
mod scene;
mod vmath;

use camera::Camera;
use clap::{App, Arg};
use image::RgbImage;
use rand::Rng;
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

    println!(
        "generating {}x{} image with {} samples per pixel",
        nx, ny, ns
    );

    // Not writing crypto so use a weak rng, also pass it around to avoid construction cost.
    let mut rng = rand::weak_rng();
    let mut scene = Scene::random_scene(&mut rng);

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

    let mut img = RgbImage::new(nx, ny);

    let start_time = SystemTime::now();

    for j in 0..ny {
        for i in 0..nx {
            let mut col = vec3(0.0, 0.0, 0.0);
            for _ in 0..ns {
                let u = (i as f32 + rng.next_f32()) / nx as f32;
                let v = ((ny - j - 1) as f32 + rng.next_f32()) / ny as f32;
                let ray = camera.get_ray(u, v, &mut rng);
                col += scene.ray_trace(&ray, 0, &mut rng);
            }
            col /= ns as f32;
            let pixel = &mut img[(i, j)];
            pixel[0] = (255.99 * col.x.sqrt()) as u8;
            pixel[1] = (255.99 * col.y.sqrt()) as u8;
            pixel[2] = (255.99 * col.z.sqrt()) as u8;
        }
    }

    let elapsed = start_time
        .elapsed()
        .expect("SystemTime elapsed time failed");
    println!(
        "{}.{:.2} seconds {} rays",
        elapsed.as_secs(),
        elapsed.subsec_nanos() * 1_000_000_000,
        scene.ray_count
    );

    img.save("output.png").expect("Failed to save output image");
}
