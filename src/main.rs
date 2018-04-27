extern crate image;
extern crate rand;

mod camera;
mod scene;
mod vmath;

use camera::Camera;
use image::RgbImage;
use rand::Rng;
use scene::Scene;
use std::f32;
use vmath::vec3;

fn main() {
    let nx = 1200;
    let ny = 800;
    let ns = 10;
    let mut rng = rand::weak_rng();
    let scene = Scene::random_scene(&mut rng);

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

    for j in 0..(ny - 1) {
        for i in 0..nx {
            let mut col = vec3(0.0, 0.0, 0.0);
            for _ in 0..ns {
                let u = (i as f32 + rng.next_f32()) / nx as f32;
                let v = ((ny - j) as f32 + rng.next_f32()) / ny as f32;
                let ray = camera.get_ray(u, v, &mut rng);
                col += scene.ray_to_colour(&ray, 0, &mut rng);
            }
            col /= ns as f32;
            let pixel = &mut img[(i, j)];
            pixel[0] = (255.99 * col.x.sqrt()) as u8;
            pixel[1] = (255.99 * col.y.sqrt()) as u8;
            pixel[2] = (255.99 * col.z.sqrt()) as u8;
        }
    }
    img.save("output.png").expect("Failed to save output image");
}
