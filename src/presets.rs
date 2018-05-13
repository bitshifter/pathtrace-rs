use camera::Camera;
use rand::Rng;
use scene::{sphere, Material, Scene};
use vmath::{Length, vec3};

pub struct Params {
    pub width: u32,
    pub height: u32,
    pub max_depth: u32,
}

pub fn from_name(name: &str, params: &Params, rng: &mut Rng) -> Option<(Scene, Camera)> {
    match name {
        "random" => Some(random(params, rng)),
        "small" => Some(small(params, rng)),
        "aras" => Some(aras_p(params, rng)),
        _ => None,
    }
}

pub fn random(params: &Params, rng: &mut Rng) -> (Scene, Camera) {
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
        Material::Lambertian {
            albedo: vec3(0.5, 0.5, 0.5),
        },
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
                    Material::Lambertian {
                        albedo: vec3(
                            rng.next_f32() * rng.next_f32(),
                            rng.next_f32() * rng.next_f32(),
                            rng.next_f32() * rng.next_f32(),
                        ),
                    },
                ));
            } else if choose_material < 0.95 {
                spheres.push(sphere(
                    centre,
                    0.2,
                    Material::Metal {
                        albedo: vec3(
                            0.5 * (1.0 + rng.next_f32()),
                            0.5 * (1.0 + rng.next_f32()),
                            0.5 * (1.0 + rng.next_f32()),
                        ),
                        fuzz: 0.5 * rng.next_f32(),
                    },
                ));
            } else {
                spheres.push(sphere(centre, 0.2, Material::Dielectric { ref_idx: 1.5 }));
            }
        }
    }
    spheres.push(sphere(
        vec3(0.0, 1.0, 0.0),
        1.0,
        Material::Dielectric { ref_idx: 1.5 },
    ));
    spheres.push(sphere(
        vec3(-4.0, 1.0, 0.0),
        1.0,
        Material::Lambertian {
            albedo: vec3(0.4, 0.2, 0.1),
        },
    ));
    spheres.push(sphere(
        vec3(4.0, 1.0, 0.0),
        1.0,
        Material::Metal {
            albedo: vec3(0.7, 0.6, 0.5),
            fuzz: 0.0,
        },
    ));

    let scene = Scene::new(&spheres, params.max_depth);
    (scene, camera)
}

pub fn small(params: &Params, _rng: &mut Rng) -> (Scene, Camera) {
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
            Material::Lambertian {
                albedo: vec3(0.1, 0.2, 0.5),
            },
        ),
        sphere(
            vec3(0.0, -100.5, -1.0),
            100.0,
            Material::Lambertian {
                albedo: vec3(0.8, 0.8, 0.0),
            },
        ),
        sphere(
            vec3(1.0, 0.0, -1.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.8, 0.6, 0.2),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-1.0, 0.0, -1.0),
            0.5,
            Material::Dielectric { ref_idx: 1.5 },
        ),
        sphere(
            vec3(-1.0, 0.0, -1.0),
            -0.45,
            Material::Dielectric { ref_idx: 1.5 },
        ),
    ];

    let scene = Scene::new(&spheres, params.max_depth);
    (scene, camera)
}

pub fn aras_p(params: &Params, _rng: &mut Rng) -> (Scene, Camera) {
    let lookfrom = vec3(0.0, 2.0, 3.0);
    let lookat = vec3(0.0, 0.0, 0.0);
    let dist_to_focus = 3.0;
    let aperture = 0.2;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        60.0,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
    );

    let spheres = [
        sphere(
            vec3(0.0, -100.5, -1.0),
            100.0,
            Material::Lambertian {
                albedo: vec3(0.8, 0.8, 0.8),
            },
        ),
        sphere(
            vec3(2.0, 0.0, -1.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.8, 0.4, 0.4),
            },
        ),
        sphere(
            vec3(0.0, 0.0, -1.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.4, 0.8, 0.4),
            },
        ),
        sphere(
            vec3(-2.0, 0.0, -1.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.4, 0.4, 0.8),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(2.0, 0.0, 1.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.4, 0.8, 0.4),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(0.0, 0.0, 1.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.4, 0.8, 0.4),
                fuzz: 0.2,
            },
        ),
        sphere(
            vec3(-2.0, 0.0, 1.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.4, 0.8, 0.4),
                fuzz: 0.6,
            },
        ),
        sphere(
            vec3(0.5, 1.0, 0.5),
            0.5,
            Material::Dielectric { ref_idx: 1.5 },
        ),
        sphere(
            vec3(-1.5, 1.5, 0.0),
            0.3,
            Material::Lambertian {
                albedo: vec3(0.8, 0.6, 0.2),
            }, // TODO: emissive vec3(30,25,15)
        ),
        sphere(
            vec3(4.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.1, 0.1, 0.1),
            },
        ),
        sphere(
            vec3(3.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.2, 0.2, 0.2),
            },
        ),
        sphere(
            vec3(2.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.3, 0.3, 0.3),
            },
        ),
        sphere(
            vec3(1.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.4, 0.4, 0.4),
            },
        ),
        sphere(
            vec3(0.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.5, 0.5, 0.5),
            },
        ),
        sphere(
            vec3(-1.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.6, 0.6, 0.6),
            },
        ),
        sphere(
            vec3(-2.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.7, 0.7, 0.7),
            },
        ),
        sphere(
            vec3(-3.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.8, 0.8, 0.8),
            },
        ),
        sphere(
            vec3(-4.0, 0.0, -3.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.9, 0.9, 0.9),
            },
        ),
        sphere(
            vec3(4.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.1, 0.1, 0.1),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(3.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.2, 0.2, 0.2),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(2.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.3, 0.3, 0.3),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(1.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.4, 0.4, 0.4),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(0.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.5, 0.5, 0.5),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-1.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.6, 0.6, 0.6),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-2.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.7, 0.7, 0.7),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-3.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.8, 0.8, 0.8),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-4.0, 0.0, -4.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.9, 0.9, 0.9),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(4.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.8, 0.1, 0.1),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(3.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.8, 0.5, 0.1),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(2.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.8, 0.8, 0.1),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(1.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.4, 0.8, 0.1),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(0.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.1, 0.8, 0.1),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-1.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.1, 0.8, 0.5),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-2.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.1, 0.8, 0.8),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-3.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.1, 0.1, 0.8),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(-4.0, 0.0, -5.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.5, 0.1, 0.8),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(4.0, 0.0, -6.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.8, 0.1, 0.1),
            },
        ),
        sphere(
            vec3(3.0, 0.0, -6.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.8, 0.5, 0.1),
            },
        ),
        sphere(
            vec3(2.0, 0.0, -6.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.8, 0.8, 0.1),
            },
        ),
        sphere(
            vec3(1.0, 0.0, -6.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.4, 0.8, 0.1),
            },
        ),
        sphere(
            vec3(0.0, 0.0, -6.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.1, 0.8, 0.1),
            },
        ),
        sphere(
            vec3(-1.0, 0.0, -6.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.1, 0.8, 0.5),
            },
        ),
        sphere(
            vec3(-2.0, 0.0, -6.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.1, 0.8, 0.8),
            },
        ),
        sphere(
            vec3(-3.0, 0.0, -6.0),
            0.5,
            Material::Lambertian {
                albedo: vec3(0.1, 0.1, 0.8),
            },
        ),
        sphere(
            vec3(-4.0, 0.0, -6.0),
            0.5,
            Material::Metal {
                albedo: vec3(0.5, 0.1, 0.8),
                fuzz: 0.0,
            },
        ),
        sphere(
            vec3(1.5, 1.5, -2.0),
            0.3,
            Material::Lambertian {
                albedo: vec3(0.1, 0.2, 0.5),
            }, // TODO: emissive vec3(3,10,20)
        ),
    ];

    let scene = Scene::new(&spheres, params.max_depth);
    (scene, camera)
}
