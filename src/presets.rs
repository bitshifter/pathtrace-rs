use crate::{
    camera::Camera,
    collision::{Hitable, MovingSphere, Sphere, XYRect},
    material,
    scene::{Params, Storage},
    texture::{self, RgbImage, Texture},
};
use glam::vec3;
use rand::Rng;
use rand_xorshift::XorShiftRng;

pub fn from_name<'a>(
    name: &str,
    params: &Params,
    rng: &mut XorShiftRng,
    storage: &'a Storage<'a>,
) -> Option<(Vec<Hitable<'a>>, Camera)> {
    println!(
        "generating '{}' preset at {}x{} with {} samples per pixel",
        name, params.width, params.height, params.samples
    );

    match name {
        "random" => Some(random(params, rng, storage)),
        "small" => Some(small(params, storage)),
        // "aras" => Some(aras_p(params, storage)),
        "smallpt" => Some(smallpt(params, storage)),
        "two_perlin_spheres" => Some(two_perlin_spheres(params, storage)),
        "simple_light" => Some(simple_light(params, storage)),
        "earth" => Some(earth(params, storage)),
        _ => None,
    }
}

pub fn random<'a>(
    params: &Params,
    rng: &mut XorShiftRng,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera) {
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
        0.0,
        1.0,
    );

    let n = 500;
    let mut hitables = Vec::with_capacity(n + 1);

    // TODO: DRY somehow
    let sphere = |centre, radius, material| -> Hitable {
        Hitable::Sphere(
            storage.alloc_sphere(Sphere::new(centre, radius)),
            storage.alloc_material(material),
        )
    };

    let moving_sphere = |centre0, centre1, radius, material| -> Hitable {
        Hitable::MovingSphere(
            storage.alloc_moving_sphere(MovingSphere::new(centre0, centre1, 0.0, 1.0, radius)),
            storage.alloc_material(material),
        )
    };

    let constant = |albedo| -> &Texture { storage.alloc_texture(texture::constant(albedo)) };
    let checker = |odd, even| -> &Texture { storage.alloc_texture(texture::checker(odd, even)) };

    hitables.push(sphere(
        vec3(0.0, -1000.0, 0.0),
        1000.0,
        material::lambertian(checker(
            constant(vec3(0.2, 0.3, 0.1)),
            constant(vec3(0.9, 0.9, 0.9)),
        )),
    ));

    for a in -11..11 {
        for b in -11..11 {
            let choose_material = rng.gen::<f32>();
            let centre = vec3(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );
            if choose_material < 0.8 {
                let centre1 = centre + vec3(0.0, 0.5 * rng.gen::<f32>(), 0.0);
                hitables.push(moving_sphere(
                    centre,
                    centre1,
                    0.2,
                    material::lambertian(constant(vec3(
                        rng.gen::<f32>() * rng.gen::<f32>(),
                        rng.gen::<f32>() * rng.gen::<f32>(),
                        rng.gen::<f32>() * rng.gen::<f32>(),
                    ))),
                ));
                // hitables.push(sphere(
                //     centre,
                //     0.2,
                //     material::lambertian(constant(vec3(
                //         rng.gen::<f32>() * rng.gen::<f32>(),
                //         rng.gen::<f32>() * rng.gen::<f32>(),
                //         rng.gen::<f32>() * rng.gen::<f32>(),
                //     ))),
            } else if choose_material < 0.95 {
                hitables.push(sphere(
                    centre,
                    0.2,
                    material::metal(
                        vec3(
                            0.5 * (1.0 + rng.gen::<f32>()),
                            0.5 * (1.0 + rng.gen::<f32>()),
                            0.5 * (1.0 + rng.gen::<f32>()),
                        ),
                        0.5 * rng.gen::<f32>(),
                    ),
                ));
            } else {
                hitables.push(sphere(centre, 0.2, material::dielectric(1.5)));
            }
        }
    }
    hitables.push(sphere(vec3(0.0, 1.0, 0.0), 1.0, material::dielectric(1.5)));
    hitables.push(sphere(
        vec3(-4.0, 1.0, 0.0),
        1.0,
        material::lambertian(constant(vec3(0.4, 0.2, 0.1))),
    ));
    hitables.push(sphere(
        vec3(4.0, 1.0, 0.0),
        1.0,
        material::metal(vec3(0.7, 0.6, 0.5), 0.0),
    ));

    // let hitable_list = Hitable::List(storage.alloc_hitables(hitables));
    // let bvh_root = BVHNode::new(rng, &mut hitables, &storage.bvhnode_arena).unwrap();
    // dbg!(bvh_root.get_stats());

    // let hitable_root = Hitable::BVHNode(bvh_root);

    // let scene = Scene::new(hitable_root);
    (hitables, camera)
}

pub fn small<'a>(params: &Params, storage: &'a Storage<'a>) -> (Vec<Hitable<'a>>, Camera) {
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
        0.0,
        1.0,
    );

    let sphere = |centre, radius, material| -> Hitable {
        Hitable::Sphere(
            storage.alloc_sphere(Sphere::new(centre, radius)),
            storage.alloc_material(material),
        )
    };

    let hitables = vec![
        sphere(
            vec3(0.0, 0.0, -1.0),
            0.5,
            material::lambertian(storage.alloc_texture(texture::constant(vec3(0.1, 0.2, 0.5)))),
        ),
        sphere(
            vec3(0.0, -100.5, -1.0),
            100.0,
            material::lambertian(storage.alloc_texture(texture::constant(vec3(0.8, 0.8, 0.0)))),
        ),
        sphere(
            vec3(1.0, 0.0, -1.0),
            0.5,
            material::metal(vec3(0.8, 0.6, 0.2), 0.0),
        ),
        sphere(vec3(-1.0, 0.0, -1.0), 0.5, material::dielectric(1.5)),
        sphere(vec3(-1.0, 0.0, -1.0), -0.45, material::dielectric(1.5)),
    ];

    (hitables, camera)
}

pub fn two_perlin_spheres<'a>(
    params: &Params,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera) {
    let lookfrom = vec3(13.0, 2.0, 3.0);
    let lookat = vec3(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        20.0,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
        0.0,
        0.0,
    );

    // TODO: DRY somehow
    let sphere = |centre, radius, material| -> Hitable {
        Hitable::Sphere(
            storage.alloc_sphere(Sphere::new(centre, radius)),
            storage.alloc_material(material),
        )
    };

    let noise_texture = storage.alloc_texture(texture::noise(&storage.perlin_noise, 4.0));

    let hitables = vec![
        sphere(
            vec3(0.0, -1000.0, 0.0),
            1000.0,
            material::lambertian(noise_texture),
        ),
        sphere(
            vec3(0.0, 2.0, 0.0),
            2.0,
            material::lambertian(noise_texture),
        ),
    ];

    (hitables, camera)
}

pub fn simple_light<'a>(params: &Params, storage: &'a Storage<'a>) -> (Vec<Hitable<'a>>, Camera) {
    let lookfrom = vec3(50.0, 2.0, 3.0);
    let lookat = vec3(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        20.0,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
        0.0,
        0.0,
    );

    // TODO: DRY somehow
    let sphere = |centre, radius, material| -> Hitable {
        Hitable::Sphere(
            storage.alloc_sphere(Sphere::new(centre, radius)),
            storage.alloc_material(material),
        )
    };

    let noise_texture = storage.alloc_texture(texture::noise(&storage.perlin_noise, 4.0));
    let constant_texture = storage.alloc_texture(texture::constant(vec3(4.0, 4.0, 4.0)));

    let hitables = vec![
        sphere(
            vec3(0.0, -1000.0, 0.0),
            1000.0,
            material::lambertian(noise_texture),
        ),
        sphere(
            vec3(0.0, 2.0, 0.0),
            2.0,
            material::lambertian(noise_texture),
        ),
        sphere(
            vec3(0.0, 7.0, 0.0),
            2.0,
            material::diffuse_light(constant_texture),
        ),
        Hitable::XYRect(
            storage.alloc_xyrect(XYRect::new(3.0, 5.0, 1.0, 3.0, -2.0)),
            storage.alloc_material(material::diffuse_light(constant_texture)),
        ),
    ];

    (hitables, camera)
}

pub fn earth<'a>(params: &Params, storage: &'a Storage<'a>) -> (Vec<Hitable<'a>>, Camera) {
    let lookfrom = vec3(13.0, 2.0, 3.0);
    let lookat = vec3(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        vec3(0.0, 1.0, 0.0),
        20.0,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
        0.0,
        0.0,
    );

    // TODO: DRY somehow
    let sphere = |centre, radius, material| -> Hitable {
        Hitable::Sphere(
            storage.alloc_sphere(Sphere::new(centre, radius)),
            storage.alloc_material(material),
        )
    };

    let earth_image = storage.alloc_image(RgbImage::open("media/earthmap.jpg"));
    let earth_texture = storage.alloc_texture(texture::rgb_image(earth_image));

    let hitables = vec![sphere(
        vec3(0.0, 0.0, 0.0),
        2.0,
        material::lambertian(earth_texture),
    )];

    (hitables, camera)
}

// pub fn aras_p<'a>(params: &Params, storage: &'a Storage<'a>) -> (Scene<'a>, Camera) {
//     let lookfrom = vec3(0.0, 2.0, 3.0);
//     let lookat = vec3(0.0, 0.0, 0.0);
//     let dist_to_focus = 3.0;
//     let aperture = 0.02;
//     let fov = 60.0;
//     let camera = Camera::new(
//         lookfrom,
//         lookat,
//         vec3(0.0, 1.0, 0.0),
//         fov,
//         params.width as f32 / params.height as f32,
//         aperture,
//         dist_to_focus,
//     );

//     let sphere = |centre, radius, material| -> (Sphere, &Material) {
//         (
//             Sphere::new(centre, radius),
//             storage.alloc_material(material),
//         )
//     };

//     let constant = |albedo| -> &Texture { storage.alloc_texture(texture::constant(albedo)) };

//     let spheres = [
//         sphere(
//             vec3(0.0, -100.5, -1.0),
//             100.0,
//             material::lambertian(constant(vec3(0.8, 0.8, 0.8))),
//         ),
//         sphere(
//             vec3(2.0, 0.0, -1.0),
//             0.5,
//             material::lambertian(constant(vec3(0.8, 0.4, 0.4))),
//         ),
//         sphere(
//             vec3(0.0, 0.0, -1.0),
//             0.5,
//             material::lambertian(constant(vec3(0.4, 0.8, 0.4))),
//         ),
//         sphere(
//             vec3(-2.0, 0.0, -1.0),
//             0.5,
//             material::metal(vec3(0.4, 0.4, 0.8), 0.0),
//         ),
//         sphere(
//             vec3(2.0, 0.0, 1.0),
//             0.5,
//             material::metal(vec3(0.4, 0.8, 0.4), 0.0),
//         ),
//         sphere(
//             vec3(0.0, 0.0, 1.0),
//             0.5,
//             material::metal(vec3(0.4, 0.8, 0.4), 0.2),
//         ),
//         sphere(
//             vec3(-2.0, 0.0, 1.0),
//             0.5,
//             material::metal(vec3(0.4, 0.8, 0.4), 0.6),
//         ),
//         sphere(vec3(0.5, 1.0, 0.5), 0.5, material::dielectric(1.5)),
//         sphere(
//             vec3(-1.5, 1.5, 0.0),
//             0.3,
//             material::diffuse_light(constant(vec3(30.0, 25.0, 15.0))),
//         ),
//         sphere(
//             vec3(4.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.1, 0.1, 0.1))),
//         ),
//         sphere(
//             vec3(3.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.2, 0.2, 0.2))),
//         ),
//         sphere(
//             vec3(2.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.3, 0.3, 0.3))),
//         ),
//         sphere(
//             vec3(1.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.4, 0.4, 0.4))),
//         ),
//         sphere(
//             vec3(0.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.5, 0.5, 0.5))),
//         ),
//         sphere(
//             vec3(-1.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.6, 0.6, 0.6))),
//         ),
//         sphere(
//             vec3(-2.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.7, 0.7, 0.7))),
//         ),
//         sphere(
//             vec3(-3.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.8, 0.8, 0.8))),
//         ),
//         sphere(
//             vec3(-4.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(vec3(0.9, 0.9, 0.9))),
//         ),
//         sphere(
//             vec3(4.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.1, 0.1, 0.1), 0.0),
//         ),
//         sphere(
//             vec3(3.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.2, 0.2, 0.2), 0.0),
//         ),
//         sphere(
//             vec3(2.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.3, 0.3, 0.3), 0.0),
//         ),
//         sphere(
//             vec3(1.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.4, 0.4, 0.4), 0.0),
//         ),
//         sphere(
//             vec3(0.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.5, 0.5, 0.5), 0.0),
//         ),
//         sphere(
//             vec3(-1.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.6, 0.6, 0.6), 0.0),
//         ),
//         sphere(
//             vec3(-2.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.7, 0.7, 0.7), 0.0),
//         ),
//         sphere(
//             vec3(-3.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.8, 0.8, 0.8), 0.0),
//         ),
//         sphere(
//             vec3(-4.0, 0.0, -4.0),
//             0.5,
//             material::metal(vec3(0.9, 0.9, 0.9), 0.0),
//         ),
//         sphere(
//             vec3(4.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.8, 0.1, 0.1), 0.0),
//         ),
//         sphere(
//             vec3(3.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.8, 0.5, 0.1), 0.0),
//         ),
//         sphere(
//             vec3(2.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.8, 0.8, 0.1), 0.0),
//         ),
//         sphere(
//             vec3(1.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.4, 0.8, 0.1), 0.0),
//         ),
//         sphere(
//             vec3(0.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.1, 0.8, 0.1), 0.0),
//         ),
//         sphere(
//             vec3(-1.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.1, 0.8, 0.5), 0.0),
//         ),
//         sphere(
//             vec3(-2.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.1, 0.8, 0.8), 0.0),
//         ),
//         sphere(
//             vec3(-3.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.1, 0.1, 0.8), 0.0),
//         ),
//         sphere(
//             vec3(-4.0, 0.0, -5.0),
//             0.5,
//             material::metal(vec3(0.5, 0.1, 0.8), 0.0),
//         ),
//         sphere(
//             vec3(4.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(vec3(0.8, 0.1, 0.1))),
//         ),
//         sphere(
//             vec3(3.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(vec3(0.8, 0.5, 0.1))),
//         ),
//         sphere(
//             vec3(2.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(vec3(0.8, 0.8, 0.1))),
//         ),
//         sphere(
//             vec3(1.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(vec3(0.4, 0.8, 0.1))),
//         ),
//         sphere(
//             vec3(0.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(vec3(0.1, 0.8, 0.1))),
//         ),
//         sphere(
//             vec3(-1.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(vec3(0.1, 0.8, 0.5))),
//         ),
//         sphere(
//             vec3(-2.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(vec3(0.1, 0.8, 0.8))),
//         ),
//         sphere(
//             vec3(-3.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(vec3(0.1, 0.1, 0.8))),
//         ),
//         sphere(
//             vec3(-4.0, 0.0, -6.0),
//             0.5,
//             material::metal(vec3(0.5, 0.1, 0.8), 0.0),
//         ),
//         sphere(
//             vec3(1.5, 1.5, -2.0),
//             0.3,
//             material::diffuse_light(constant(vec3(3.0, 10.0, 20.0))),
//         ),
//     ];

//     let scene = Scene::new(&spheres);
//     (scene, camera)
// }

pub fn smallpt<'a>(params: &Params, storage: &'a Storage<'a>) -> (Vec<Hitable<'a>>, Camera) {
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
        0.0,
        1.0,
    );

    let sphere = |centre, radius, material| -> Hitable {
        Hitable::Sphere(
            storage.alloc_sphere(Sphere::new(centre, radius)),
            storage.alloc_material(material),
        )
    };

    let constant = |albedo| -> &Texture { storage.alloc_texture(texture::constant(albedo)) };

    let hitables = vec![
        sphere(
            vec3(1e3 + 1.0, 40.8, 81.6),
            1e3,
            material::lambertian(constant(vec3(0.75, 0.25, 0.25))),
        ), //Left
        sphere(
            vec3(-1e3 + 99.0, 40.8, 81.6),
            1e3,
            material::lambertian(constant(vec3(0.25, 0.25, 0.75))),
        ), //Rght
        sphere(
            vec3(50.0, 40.8, 1e3),
            1e3,
            material::lambertian(constant(vec3(0.75, 0.75, 0.75))),
        ), //Back
        // sphere(
        //     vec3(50.0, 40.8, -1e3 + 300.0),
        //     1e3,
        //     material::lambertian {
        //         albedo: vec3(0.1, 0.1, 0.1),
        //     },
        // ), //Frnt
        sphere(
            vec3(50.0, 1e3, 81.6),
            1e3,
            material::lambertian(constant(vec3(0.75, 0.75, 0.75))),
        ), //Botm
        sphere(
            vec3(50.0, -1e3 + 81.6, 81.6),
            1e3,
            material::lambertian(constant(vec3(0.75, 0.75, 0.75))),
        ), //Top
        sphere(
            vec3(27.0, 16.5, 47.0),
            16.5,
            material::metal(vec3(1.0, 1.0, 1.0) * 0.999, 0.0),
        ), //Mirr
        sphere(vec3(73.0, 16.5, 78.0), 16.5, material::dielectric(1.5)), //Glas
        sphere(
            vec3(50.0, 81.6 - 16.5, 81.6),
            1.5,
            material::diffuse_light(constant(vec3(4.0, 4.0, 4.0) * 100.0)),
        ), //Lite
    ];

    (hitables, camera)
}
