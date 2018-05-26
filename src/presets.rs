extern crate rand;

use camera::Camera;
use collision::sphere;
use material::MaterialKind;
use rand::{Rng, SeedableRng, XorShiftRng};
use scene::{Params, Scene};
use vmath::{vec3, Length};

pub fn from_name(name: &str, params: &Params) -> Option<(Scene, Camera)> {
    match name {
        "random" => Some(random(params)),
        "small" => Some(small(params)),
        "aras" => Some(aras_p(params)),
        "smallpt" => Some(smallpt(params)),
        _ => None,
    }
}

pub fn random(params: &Params) -> (Scene, Camera) {
    let mut rng = if params.random_seed {
        rand::weak_rng()
    } else {
        const FIXED_SEED: [u32; 4] = [0x193a_6754, 0xa8a7_d469, 0x9783_0e05, 0x113b_a7bb];
        XorShiftRng::from_seed(FIXED_SEED)
    };

    let lookfrom = vec3(13.0, 2.0, 3.0);
    let lookat = vec3(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        20.0,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
    );

    let n = 500;
    let mut spheres = Vec::with_capacity(n + 1);
    spheres.push(sphere(
        vec3(0.0, -1000.0, 0.0),
        1000.0,
        MaterialKind::Lambertian {
            albedo: vec3(0.5, 0.5, 0.5),
        },
        None,
    ));
    for a in -11..11 {
        for b in -11..11 {
            let choose_material = rng.next_f32();
            let centre = vec3(
                a as f32 + 0.9 * rng.next_f32(),
                0.2,
                b as f32 + 0.9 * rng.next_f32(),
            );
            if choose_material < 0.8 {
                spheres.push(sphere(
                    centre,
                    0.2,
                    MaterialKind::Lambertian {
                        albedo: vec3(
                            rng.next_f32() * rng.next_f32(),
                            rng.next_f32() * rng.next_f32(),
                            rng.next_f32() * rng.next_f32(),
                        ),
                    },
                    None,
                ));
            } else if choose_material < 0.95 {
                spheres.push(sphere(
                    centre,
                    0.2,
                    MaterialKind::Metal {
                        albedo: vec3(
                            0.5 * (1.0 + rng.next_f32()),
                            0.5 * (1.0 + rng.next_f32()),
                            0.5 * (1.0 + rng.next_f32()),
                        ),
                        fuzz: 0.5 * rng.next_f32(),
                    },
                    None,
                ));
            } else {
                spheres.push(sphere(
                    centre,
                    0.2,
                    MaterialKind::Dielectric { ref_idx: 1.5 },
                    None,
                ));
            }
        }
    }
    spheres.push(sphere(
        vec3(0.0, 1.0, 0.0),
        1.0,
        MaterialKind::Dielectric { ref_idx: 1.5 },
        None,
    ));
    spheres.push(sphere(
        vec3(-4.0, 1.0, 0.0),
        1.0,
        MaterialKind::Lambertian {
            albedo: vec3(0.4, 0.2, 0.1),
        },
        None,
    ));
    spheres.push(sphere(
        vec3(4.0, 1.0, 0.0),
        1.0,
        MaterialKind::Metal {
            albedo: vec3(0.7, 0.6, 0.5),
            fuzz: 0.0,
        },
        None,
    ));

    let scene = Scene::new(&spheres);
    (scene, camera)
}

pub fn small(params: &Params) -> (Scene, Camera) {
    let lookfrom = vec3(3.0, 3.0, 2.0);
    let lookat = vec3(0.0, 0.0, -1.0);
    let dist_to_focus = (lookfrom - lookat).length();
    let aperture = 0.1;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        20.0,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
    );

    let spheres = [
        sphere(
            vec3(0.0, 0.0, -1.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.1, 0.2, 0.5),
            },
            None,
        ),
        sphere(
            vec3(0.0, -100.5, -1.0),
            100.0,
            MaterialKind::Lambertian {
                albedo: vec3(0.8, 0.8, 0.0),
            },
            None,
        ),
        sphere(
            vec3(1.0, 0.0, -1.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.8, 0.6, 0.2),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-1.0, 0.0, -1.0),
            0.5,
            MaterialKind::Dielectric { ref_idx: 1.5 },
            None,
        ),
        sphere(
            vec3(-1.0, 0.0, -1.0),
            -0.45,
            MaterialKind::Dielectric { ref_idx: 1.5 },
            None,
        ),
    ];

    let scene = Scene::new(&spheres);
    (scene, camera)
}

pub fn aras_p(params: &Params) -> (Scene, Camera) {
    let lookfrom = vec3(0.0, 2.0, 3.0);
    let lookat = vec3(0.0, 0.0, 0.0);
    let dist_to_focus = 3.0;
    let aperture = 0.02;
    let fov = 60.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        fov,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
    );

    let spheres = [
        sphere(
            vec3(0.0, -100.5, -1.0),
            100.0,
            MaterialKind::Lambertian {
                albedo: vec3(0.8, 0.8, 0.8),
            },
            None,
        ),
        sphere(
            vec3(2.0, 0.0, -1.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.8, 0.4, 0.4),
            },
            None,
        ),
        sphere(
            vec3(0.0, 0.0, -1.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.4, 0.8, 0.4),
            },
            None,
        ),
        sphere(
            vec3(-2.0, 0.0, -1.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.4, 0.4, 0.8),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(2.0, 0.0, 1.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.4, 0.8, 0.4),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(0.0, 0.0, 1.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.4, 0.8, 0.4),
                fuzz: 0.2,
            },
            None,
        ),
        sphere(
            vec3(-2.0, 0.0, 1.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.4, 0.8, 0.4),
                fuzz: 0.6,
            },
            None,
        ),
        sphere(
            vec3(0.5, 1.0, 0.5),
            0.5,
            MaterialKind::Dielectric { ref_idx: 1.5 },
            None,
        ),
        sphere(
            vec3(-1.5, 1.5, 0.0),
            0.3,
            MaterialKind::Lambertian {
                albedo: vec3(0.8, 0.6, 0.2),
            },
            Some(vec3(30.0, 25.0, 15.0)),
        ),
        sphere(
            vec3(4.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.1, 0.1, 0.1),
            },
            None,
        ),
        sphere(
            vec3(3.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.2, 0.2, 0.2),
            },
            None,
        ),
        sphere(
            vec3(2.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.3, 0.3, 0.3),
            },
            None,
        ),
        sphere(
            vec3(1.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.4, 0.4, 0.4),
            },
            None,
        ),
        sphere(
            vec3(0.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.5, 0.5, 0.5),
            },
            None,
        ),
        sphere(
            vec3(-1.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.6, 0.6, 0.6),
            },
            None,
        ),
        sphere(
            vec3(-2.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.7, 0.7, 0.7),
            },
            None,
        ),
        sphere(
            vec3(-3.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.8, 0.8, 0.8),
            },
            None,
        ),
        sphere(
            vec3(-4.0, 0.0, -3.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.9, 0.9, 0.9),
            },
            None,
        ),
        sphere(
            vec3(4.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.1, 0.1, 0.1),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(3.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.2, 0.2, 0.2),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(2.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.3, 0.3, 0.3),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(1.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.4, 0.4, 0.4),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(0.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.5, 0.5, 0.5),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-1.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.6, 0.6, 0.6),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-2.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.7, 0.7, 0.7),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-3.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.8, 0.8, 0.8),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-4.0, 0.0, -4.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.9, 0.9, 0.9),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(4.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.8, 0.1, 0.1),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(3.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.8, 0.5, 0.1),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(2.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.8, 0.8, 0.1),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(1.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.4, 0.8, 0.1),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(0.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.1, 0.8, 0.1),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-1.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.1, 0.8, 0.5),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-2.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.1, 0.8, 0.8),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-3.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.1, 0.1, 0.8),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(-4.0, 0.0, -5.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.5, 0.1, 0.8),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(4.0, 0.0, -6.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.8, 0.1, 0.1),
            },
            None,
        ),
        sphere(
            vec3(3.0, 0.0, -6.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.8, 0.5, 0.1),
            },
            None,
        ),
        sphere(
            vec3(2.0, 0.0, -6.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.8, 0.8, 0.1),
            },
            None,
        ),
        sphere(
            vec3(1.0, 0.0, -6.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.4, 0.8, 0.1),
            },
            None,
        ),
        sphere(
            vec3(0.0, 0.0, -6.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.1, 0.8, 0.1),
            },
            None,
        ),
        sphere(
            vec3(-1.0, 0.0, -6.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.1, 0.8, 0.5),
            },
            None,
        ),
        sphere(
            vec3(-2.0, 0.0, -6.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.1, 0.8, 0.8),
            },
            None,
        ),
        sphere(
            vec3(-3.0, 0.0, -6.0),
            0.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.1, 0.1, 0.8),
            },
            None,
        ),
        sphere(
            vec3(-4.0, 0.0, -6.0),
            0.5,
            MaterialKind::Metal {
                albedo: vec3(0.5, 0.1, 0.8),
                fuzz: 0.0,
            },
            None,
        ),
        sphere(
            vec3(1.5, 1.5, -2.0),
            0.3,
            MaterialKind::Lambertian {
                albedo: vec3(0.1, 0.2, 0.5),
            },
            Some(vec3(3.0, 10.0, 20.0)),
        ),
    ];

    let scene = Scene::new(&spheres);
    (scene, camera)
}

pub fn smallpt(params: &Params) -> (Scene, Camera) {
    let lookfrom = vec3(50.0, 52.0, 295.6);
    let lookat = vec3(50.0, 33.0, 0.0);
    let dist_to_focus = 100.0;
    let aperture = 0.05;
    let fov = 30.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        fov,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
    );

    let spheres = [
        sphere(
            vec3(1e3 + 1.0, 40.8, 81.6),
            1e3,
            MaterialKind::Lambertian {
                albedo: vec3(0.75, 0.25, 0.25),
            },
            None,
        ), //Left
        sphere(
            vec3(-1e3 + 99.0, 40.8, 81.6),
            1e3,
            MaterialKind::Lambertian {
                albedo: vec3(0.25, 0.25, 0.75),
            },
            None,
        ), //Rght
        sphere(
            vec3(50.0, 40.8, 1e3),
            1e3,
            MaterialKind::Lambertian {
                albedo: vec3(0.75, 0.75, 0.75),
            },
            None,
        ), //Back
        // sphere(
        //     vec3(50.0, 40.8, -1e3 + 300.0),
        //     1e3,
        //     MaterialKind::Lambertian {
        //         albedo: vec3(0.1, 0.1, 0.1),
        //     },
        //     None,
        // ), //Frnt
        sphere(
            vec3(50.0, 1e3, 81.6),
            1e3,
            MaterialKind::Lambertian {
                albedo: vec3(0.75, 0.75, 0.75),
            },
            None,
        ), //Botm
        sphere(
            vec3(50.0, -1e3 + 81.6, 81.6),
            1e3,
            MaterialKind::Lambertian {
                albedo: vec3(0.75, 0.75, 0.75),
            },
            None,
        ), //Top
        sphere(
            vec3(27.0, 16.5, 47.0),
            16.5,
            MaterialKind::Metal {
                albedo: vec3(1.0, 1.0, 1.0) * 0.999,
                fuzz: 0.0,
            },
            None,
        ), //Mirr
        sphere(
            vec3(73.0, 16.5, 78.0),
            16.5,
            MaterialKind::Dielectric { ref_idx: 1.5 },
            None,
        ), //Glas
        sphere(
            vec3(50.0, 81.6 - 16.5, 81.6),
            1.5,
            MaterialKind::Lambertian {
                albedo: vec3(0.0, 0.0, 0.0),
            },
            Some(vec3(4.0, 4.0, 4.0) * 100.0),
        ), //Lite
    ];

    let scene = Scene::new(&spheres);
    (scene, camera)
}
