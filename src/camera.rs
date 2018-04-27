extern crate rand;
extern crate std;

use rand::Rng;
use std::f32;
use vmath::{cross, dot, normalize, ray, Ray, Vec3, vec3};

fn random_in_unit_disk(rng: &mut Rng) -> Vec3 {
    loop {
        let p = 2.0 * vec3(rng.next_f32(), rng.next_f32(), 0.0) - vec3(1.0, 1.0, 0.0);
        if dot(p, p) < 1.0 {
            return p;
        }
    }
}

pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: f32,
}

impl Camera {
    pub fn new(
        lookfrom: Vec3,
        lookat: Vec3,
        vup: Vec3,
        vfov: f32,
        aspect: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Camera {
        let theta = vfov * f32::consts::PI / 180.0;
        let half_height = (theta * 0.5).tan();
        let half_width = aspect * half_height;
        let w = normalize(lookfrom - lookat);
        let u = normalize(cross(vup, w));
        let v = cross(w, u);
        Camera {
            origin: lookfrom,
            lower_left_corner: lookfrom - half_width * focus_dist * u - half_height * focus_dist * v
                - focus_dist * w,
            horizontal: 2.0 * half_width * focus_dist * u,
            vertical: 2.0 * half_height * focus_dist * v,
            u: u,
            v: v,
            lens_radius: aperture * 0.5,
        }
    }

    pub fn get_ray(&self, s: f32, t: f32, rng: &mut Rng) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk(rng);
        let offset = self.u * rd.x + self.v * rd.y;
        ray(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}
