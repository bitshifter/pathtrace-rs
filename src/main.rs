extern crate image;
extern crate rand;

mod camera;
mod scene;
mod vmath;

use camera::Camera;
use image::RgbImage;
use rand::Rng;
use scene::Scene;

use vmath::{sphere, vec3};

fn main() {
    let nx = 200;
    let ny = 100;
    let ns = 100;
    let scene = Scene::new(vec![
        sphere(vec3(0.0, 0.0, -1.0), 0.5),
        sphere(vec3(0.0, -100.5, -1.0), 100.0),
    ]);
    let camera = Camera::default();
    let mut rng = rand::weak_rng();
    let mut img = RgbImage::new(nx, ny);
    for j in 0..(ny - 1) {
        for i in 0..nx {
            let mut col = vec3(0.0, 0.0, 0.0);
            for _ in 0..ns {
                let u = (i as f32 + rng.next_f32()) / nx as f32;
                let v = ((ny - j) as f32 + rng.next_f32()) / ny as f32;
                let ray = camera.get_ray(u, v);
                col += scene.ray_to_colour(&ray, &mut rng);
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
