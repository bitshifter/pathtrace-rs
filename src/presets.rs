use crate::{
    camera::Camera,
    collision::{ConstantMedium, Cuboid, Hitable, Instance, MovingSphere, Rect, Sphere},
    material,
    params::Params,
    storage::Storage,
    texture::{self, RgbImage, Texture},
};
use glam::{Angle, Mat4, Quat, Vec3};
use rand::Rng;
use rand_xoshiro::Xoshiro256Plus;

pub fn from_name<'a>(
    name: &str,
    params: &Params,
    rng: &mut Xoshiro256Plus,
    storage: &'a Storage<'a>,
) -> Option<(Vec<Hitable<'a>>, Camera, Option<Vec3>)> {
    println!(
        "generating '{}' preset at {}x{} with {} samples per pixel",
        name, params.width, params.height, params.samples
    );

    match name {
        "random" => Some(random(params, rng, storage)),
        "random_spheres" => Some(random_spheres(params, rng, storage)),
        "small" => Some(small(params, storage)),
        // "aras" => Some(aras_p(params, storage)),
        "smallpt" => Some(smallpt(params, storage)),
        "cornell" => Some(cornell_box(params, storage)),
        "cornell_smoke" => Some(cornell_smoke(params, storage)),
        "two_perlin_spheres" => Some(two_perlin_spheres(params, storage)),
        "simple_light" => Some(simple_light(params, storage)),
        "earth" => Some(earth(params, storage)),
        "final" => Some(final_scene(params, rng, storage)),
        _ => None,
    }
}

pub fn final_scene<'a>(
    params: &Params,
    rng: &mut Xoshiro256Plus,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
        0.0,
        1.0,
    );
    let n = 500;
    let mut hitables = Vec::with_capacity(n + 1);
    // let mut boxes1 = Vec::with_capacity(10000);
    // let mut boxes2 = Vec::with_capacity(10000);

    let white =
        material::lambertian(storage.alloc_texture(texture::constant(Vec3::new(0.73, 0.73, 0.73))));
    let ground =
        material::lambertian(storage.alloc_texture(texture::constant(Vec3::new(0.48, 0.83, 0.53))));

    (hitables, camera, None)
}

pub fn random<'a>(
    params: &Params,
    rng: &mut Xoshiro256Plus,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    random_impl(params, false, rng, storage)
}

pub fn random_spheres<'a>(
    params: &Params,
    rng: &mut Xoshiro256Plus,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    random_impl(params, true, rng, storage)
}

fn random_impl<'a>(
    params: &Params,
    only_spheres: bool,
    rng: &mut Xoshiro256Plus,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
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
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        material::lambertian(checker(
            constant(Vec3::new(0.2, 0.3, 0.1)),
            constant(Vec3::new(0.9, 0.9, 0.9)),
        )),
    ));

    for a in -11..11 {
        for b in -11..11 {
            let choose_material = rng.gen::<f32>();
            let centre = Vec3::new(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );
            if choose_material < 0.8 {
                let centre1 = centre + Vec3::new(0.0, 0.5 * rng.gen::<f32>(), 0.0);
                if only_spheres {
                    hitables.push(sphere(
                        centre,
                        0.2,
                        material::lambertian(constant(Vec3::new(
                            rng.gen::<f32>() * rng.gen::<f32>(),
                            rng.gen::<f32>() * rng.gen::<f32>(),
                            rng.gen::<f32>() * rng.gen::<f32>(),
                        ))),
                    ));
                } else {
                    hitables.push(moving_sphere(
                        centre,
                        centre1,
                        0.2,
                        material::lambertian(constant(Vec3::new(
                            rng.gen::<f32>() * rng.gen::<f32>(),
                            rng.gen::<f32>() * rng.gen::<f32>(),
                            rng.gen::<f32>() * rng.gen::<f32>(),
                        ))),
                    ));
                }
            } else if choose_material < 0.95 {
                hitables.push(sphere(
                    centre,
                    0.2,
                    material::metal(
                        Vec3::new(
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
    hitables.push(sphere(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        material::dielectric(1.5),
    ));
    hitables.push(sphere(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        material::lambertian(constant(Vec3::new(0.4, 0.2, 0.1))),
    ));
    hitables.push(sphere(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        material::metal(Vec3::new(0.7, 0.6, 0.5), 0.0),
    ));

    // let hitable_list = Hitable::List(storage.alloc_hitables(hitables));
    // let bvh_root = BVHNode::new(rng, &mut hitables, &storage.bvhnode_arena).unwrap();
    // dbg!(bvh_root.get_stats());

    // let hitable_root = Hitable::BVHNode(bvh_root);

    // let scene = Scene::new(hitable_root);
    (hitables, camera, None)
}

pub fn small<'a>(
    params: &Params,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(3.0, 3.0, 2.0);
    let lookat = Vec3::new(0.0, 0.0, -1.0);
    let dist_to_focus = (lookfrom - lookat).length();
    let aperture = 0.1;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
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
            Vec3::new(0.0, 0.0, -1.0),
            0.5,
            material::lambertian(
                storage.alloc_texture(texture::constant(Vec3::new(0.1, 0.2, 0.5))),
            ),
        ),
        sphere(
            Vec3::new(0.0, -100.5, -1.0),
            100.0,
            material::lambertian(
                storage.alloc_texture(texture::constant(Vec3::new(0.8, 0.8, 0.0))),
            ),
        ),
        sphere(
            Vec3::new(1.0, 0.0, -1.0),
            0.5,
            material::metal(Vec3::new(0.8, 0.6, 0.2), 0.0),
        ),
        sphere(Vec3::new(-1.0, 0.0, -1.0), 0.5, material::dielectric(1.5)),
        sphere(Vec3::new(-1.0, 0.0, -1.0), -0.45, material::dielectric(1.5)),
    ];

    (hitables, camera, None)
}

pub fn two_perlin_spheres<'a>(
    params: &Params,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
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
            Vec3::new(0.0, -1000.0, 0.0),
            1000.0,
            material::lambertian(noise_texture),
        ),
        sphere(
            Vec3::new(0.0, 2.0, 0.0),
            2.0,
            material::lambertian(noise_texture),
        ),
    ];

    (hitables, camera, None)
}

pub fn simple_light<'a>(
    params: &Params,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(50.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
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
    let constant_texture = storage.alloc_texture(texture::constant(Vec3::new(4.0, 4.0, 4.0)));

    let hitables = vec![
        sphere(
            Vec3::new(0.0, -1000.0, 0.0),
            1000.0,
            material::lambertian(noise_texture),
        ),
        sphere(
            Vec3::new(0.0, 2.0, 0.0),
            2.0,
            material::lambertian(noise_texture),
        ),
        sphere(
            Vec3::new(0.0, 7.0, 0.0),
            2.0,
            material::diffuse_light(constant_texture),
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xy(3.0, 5.0, 1.0, 3.0, -2.0, false)),
            storage.alloc_material(material::diffuse_light(constant_texture)),
        ),
    ];

    (hitables, camera, Some(Vec3::zero()))
}

pub fn cornell_box<'a>(
    params: &Params,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(278.0, 278.0, -800.0);
    let lookat = Vec3::new(278.0, 278.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;
    let vfov = 40.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
        vfov,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
        0.0,
        1.0,
    );

    let red = storage.alloc_material(material::lambertian(
        storage.alloc_texture(texture::constant(Vec3::new(0.65, 0.05, 0.05))),
    ));
    let white = storage.alloc_material(material::lambertian(
        storage.alloc_texture(texture::constant(Vec3::new(0.73, 0.73, 0.73))),
    ));
    let green = storage.alloc_material(material::lambertian(
        storage.alloc_texture(texture::constant(Vec3::new(0.12, 0.45, 0.15))),
    ));
    let light = storage.alloc_material(material::diffuse_light(
        storage.alloc_texture(texture::constant(Vec3::new(15.0, 15.0, 15.0))),
    ));

    let box1_transform = Mat4::from_rotation_translation(
        Quat::from_rotation_y(Angle::from_degrees(-18.0)),
        Vec3::new(130.0, 0.0, 65.0),
    );
    let box2_transform = Mat4::from_rotation_translation(
        Quat::from_rotation_y(Angle::from_degrees(15.0)),
        Vec3::new(265.0, 0.0, 295.0),
    );

    let hitables = vec![
        Hitable::Rect(
            storage.alloc_rect(Rect::new_yz(0.0, 555.0, 0.0, 555.0, 555.0, true)),
            green,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_yz(0.0, 555.0, 0.0, 555.0, 0.0, false)),
            red,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xz(213.0, 343.0, 227.0, 332.0, 554.0, false)),
            light,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xz(0.0, 555.0, 0.0, 555.0, 555.0, true)),
            white,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xz(0.0, 555.0, 0.0, 555.0, 0.0, false)),
            white,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xy(0.0, 555.0, 0.0, 555.0, 555.0, true)),
            white,
        ),
        Hitable::Instance(storage.alloc_instance(Instance::new(
            Hitable::Cuboid(
                storage.alloc_cuboid(Cuboid::new(Vec3::zero(), Vec3::new(165.0, 165.0, 165.0))),
                white,
            ),
            box1_transform,
        ))),
        Hitable::Instance(storage.alloc_instance(Instance::new(
            Hitable::Cuboid(
                storage.alloc_cuboid(Cuboid::new(Vec3::zero(), Vec3::new(165.0, 330.0, 165.0))),
                white,
            ),
            box2_transform,
        ))),
    ];

    (hitables, camera, Some(Vec3::zero()))
}

pub fn cornell_smoke<'a>(
    params: &Params,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(278.0, 278.0, -800.0);
    let lookat = Vec3::new(278.0, 278.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;
    let vfov = 40.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
        vfov,
        params.width as f32 / params.height as f32,
        aperture,
        dist_to_focus,
        0.0,
        1.0,
    );

    let red = storage.alloc_material(material::lambertian(
        storage.alloc_texture(texture::constant(Vec3::new(0.65, 0.05, 0.05))),
    ));
    let white = storage.alloc_material(material::lambertian(
        storage.alloc_texture(texture::constant(Vec3::new(0.73, 0.73, 0.73))),
    ));
    let green = storage.alloc_material(material::lambertian(
        storage.alloc_texture(texture::constant(Vec3::new(0.12, 0.45, 0.15))),
    ));
    let light = storage.alloc_material(material::diffuse_light(
        storage.alloc_texture(texture::constant(Vec3::new(7.0, 7.0, 7.0))),
    ));

    let box1_transform = Mat4::from_rotation_translation(
        Quat::from_rotation_y(Angle::from_degrees(-18.0)),
        Vec3::new(130.0, 0.0, 65.0),
    );
    let box2_transform = Mat4::from_rotation_translation(
        Quat::from_rotation_y(Angle::from_degrees(15.0)),
        Vec3::new(265.0, 0.0, 295.0),
    );

    let hitables = vec![
        Hitable::Rect(
            storage.alloc_rect(Rect::new_yz(0.0, 555.0, 0.0, 555.0, 555.0, true)),
            green,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_yz(0.0, 555.0, 0.0, 555.0, 0.0, false)),
            red,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xz(113.0, 443.0, 127.0, 432.0, 554.0, false)),
            light,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xz(0.0, 555.0, 0.0, 555.0, 555.0, true)),
            white,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xz(0.0, 555.0, 0.0, 555.0, 0.0, false)),
            white,
        ),
        Hitable::Rect(
            storage.alloc_rect(Rect::new_xy(0.0, 555.0, 0.0, 555.0, 555.0, true)),
            white,
        ),
        Hitable::ConstantMedium(storage.alloc_constant_medium(ConstantMedium::new(
            Hitable::Instance(storage.alloc_instance(Instance::new(
                Hitable::Cuboid(
                    storage.alloc_cuboid(Cuboid::new(Vec3::zero(), Vec3::new(165.0, 165.0, 165.0))),
                    white,
                ),
                box1_transform,
            ))),
            0.01,
            storage.alloc_texture(texture::constant(Vec3::one())),
        ))),
        Hitable::ConstantMedium(storage.alloc_constant_medium(ConstantMedium::new(
            Hitable::Instance(storage.alloc_instance(Instance::new(
                Hitable::Cuboid(
                    storage.alloc_cuboid(Cuboid::new(Vec3::zero(), Vec3::new(165.0, 330.0, 165.0))),
                    white,
                ),
                box2_transform,
            ))),
            0.01,
            storage.alloc_texture(texture::constant(Vec3::zero())),
        ))),
    ];

    (hitables, camera, Some(Vec3::zero()))
}

pub fn earth<'a>(
    params: &Params,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(13.0, 2.0, 3.0);
    let lookat = Vec3::new(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
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
        Vec3::new(0.0, 0.0, 0.0),
        2.0,
        material::lambertian(earth_texture),
    )];

    (hitables, camera, None)
}

// pub fn aras_p<'a>(params: &Params, storage: &'a Storage<'a>) -> (Scene<'a>, Camera, Option<Vec3>) {
//     let lookfrom = Vec3::new(0.0, 2.0, 3.0);
//     let lookat = Vec3::new(0.0, 0.0, 0.0);
//     let dist_to_focus = 3.0;
//     let aperture = 0.02;
//     let fov = 60.0;
//     let camera = Camera::new(
//         lookfrom,
//         lookat,
//         Vec3::new(0.0, 1.0, 0.0),
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
//             Vec3::new(0.0, -100.5, -1.0),
//             100.0,
//             material::lambertian(constant(Vec3::new(0.8, 0.8, 0.8))),
//         ),
//         sphere(
//             Vec3::new(2.0, 0.0, -1.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.8, 0.4, 0.4))),
//         ),
//         sphere(
//             Vec3::new(0.0, 0.0, -1.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.4, 0.8, 0.4))),
//         ),
//         sphere(
//             Vec3::new(-2.0, 0.0, -1.0),
//             0.5,
//             material::metal(Vec3::new(0.4, 0.4, 0.8), 0.0),
//         ),
//         sphere(
//             Vec3::new(2.0, 0.0, 1.0),
//             0.5,
//             material::metal(Vec3::new(0.4, 0.8, 0.4), 0.0),
//         ),
//         sphere(
//             Vec3::new(0.0, 0.0, 1.0),
//             0.5,
//             material::metal(Vec3::new(0.4, 0.8, 0.4), 0.2),
//         ),
//         sphere(
//             Vec3::new(-2.0, 0.0, 1.0),
//             0.5,
//             material::metal(Vec3::new(0.4, 0.8, 0.4), 0.6),
//         ),
//         sphere(Vec3::new(0.5, 1.0, 0.5), 0.5, material::dielectric(1.5)),
//         sphere(
//             Vec3::new(-1.5, 1.5, 0.0),
//             0.3,
//             material::diffuse_light(constant(Vec3::new(30.0, 25.0, 15.0))),
//         ),
//         sphere(
//             Vec3::new(4.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.1, 0.1, 0.1))),
//         ),
//         sphere(
//             Vec3::new(3.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.2, 0.2, 0.2))),
//         ),
//         sphere(
//             Vec3::new(2.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.3, 0.3, 0.3))),
//         ),
//         sphere(
//             Vec3::new(1.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.4, 0.4, 0.4))),
//         ),
//         sphere(
//             Vec3::new(0.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.5, 0.5, 0.5))),
//         ),
//         sphere(
//             Vec3::new(-1.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.6, 0.6, 0.6))),
//         ),
//         sphere(
//             Vec3::new(-2.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.7, 0.7, 0.7))),
//         ),
//         sphere(
//             Vec3::new(-3.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.8, 0.8, 0.8))),
//         ),
//         sphere(
//             Vec3::new(-4.0, 0.0, -3.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.9, 0.9, 0.9))),
//         ),
//         sphere(
//             Vec3::new(4.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.1, 0.1, 0.1), 0.0),
//         ),
//         sphere(
//             Vec3::new(3.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.2, 0.2, 0.2), 0.0),
//         ),
//         sphere(
//             Vec3::new(2.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.3, 0.3, 0.3), 0.0),
//         ),
//         sphere(
//             Vec3::new(1.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.4, 0.4, 0.4), 0.0),
//         ),
//         sphere(
//             Vec3::new(0.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.5, 0.5, 0.5), 0.0),
//         ),
//         sphere(
//             Vec3::new(-1.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.6, 0.6, 0.6), 0.0),
//         ),
//         sphere(
//             Vec3::new(-2.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.7, 0.7, 0.7), 0.0),
//         ),
//         sphere(
//             Vec3::new(-3.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.8, 0.8, 0.8), 0.0),
//         ),
//         sphere(
//             Vec3::new(-4.0, 0.0, -4.0),
//             0.5,
//             material::metal(Vec3::new(0.9, 0.9, 0.9), 0.0),
//         ),
//         sphere(
//             Vec3::new(4.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.8, 0.1, 0.1), 0.0),
//         ),
//         sphere(
//             Vec3::new(3.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.8, 0.5, 0.1), 0.0),
//         ),
//         sphere(
//             Vec3::new(2.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.8, 0.8, 0.1), 0.0),
//         ),
//         sphere(
//             Vec3::new(1.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.4, 0.8, 0.1), 0.0),
//         ),
//         sphere(
//             Vec3::new(0.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.1, 0.8, 0.1), 0.0),
//         ),
//         sphere(
//             Vec3::new(-1.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.1, 0.8, 0.5), 0.0),
//         ),
//         sphere(
//             Vec3::new(-2.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.1, 0.8, 0.8), 0.0),
//         ),
//         sphere(
//             Vec3::new(-3.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.1, 0.1, 0.8), 0.0),
//         ),
//         sphere(
//             Vec3::new(-4.0, 0.0, -5.0),
//             0.5,
//             material::metal(Vec3::new(0.5, 0.1, 0.8), 0.0),
//         ),
//         sphere(
//             Vec3::new(4.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.8, 0.1, 0.1))),
//         ),
//         sphere(
//             Vec3::new(3.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.8, 0.5, 0.1))),
//         ),
//         sphere(
//             Vec3::new(2.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.8, 0.8, 0.1))),
//         ),
//         sphere(
//             Vec3::new(1.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.4, 0.8, 0.1))),
//         ),
//         sphere(
//             Vec3::new(0.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.1, 0.8, 0.1))),
//         ),
//         sphere(
//             Vec3::new(-1.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.1, 0.8, 0.5))),
//         ),
//         sphere(
//             Vec3::new(-2.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.1, 0.8, 0.8))),
//         ),
//         sphere(
//             Vec3::new(-3.0, 0.0, -6.0),
//             0.5,
//             material::lambertian(constant(Vec3::new(0.1, 0.1, 0.8))),
//         ),
//         sphere(
//             Vec3::new(-4.0, 0.0, -6.0),
//             0.5,
//             material::metal(Vec3::new(0.5, 0.1, 0.8), 0.0),
//         ),
//         sphere(
//             Vec3::new(1.5, 1.5, -2.0),
//             0.3,
//             material::diffuse_light(constant(Vec3::new(3.0, 10.0, 20.0))),
//         ),
//     ];

//     let scene = Scene::new(&spheres);
//     (scene, camera)
// }

pub fn smallpt<'a>(
    params: &Params,
    storage: &'a Storage<'a>,
) -> (Vec<Hitable<'a>>, Camera, Option<Vec3>) {
    let lookfrom = Vec3::new(50.0, 52.0, 295.6);
    let lookat = Vec3::new(50.0, 33.0, 0.0);
    let dist_to_focus = 100.0;
    let aperture = 0.05;
    let fov = 30.0;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
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
            Vec3::new(1e3 + 1.0, 40.8, 81.6),
            1e3,
            material::lambertian(constant(Vec3::new(0.75, 0.25, 0.25))),
        ), //Left
        sphere(
            Vec3::new(-1e3 + 99.0, 40.8, 81.6),
            1e3,
            material::lambertian(constant(Vec3::new(0.25, 0.25, 0.75))),
        ), //Rght
        sphere(
            Vec3::new(50.0, 40.8, 1e3),
            1e3,
            material::lambertian(constant(Vec3::new(0.75, 0.75, 0.75))),
        ), //Back
        // sphere(
        //     Vec3::new(50.0, 40.8, -1e3 + 300.0),
        //     1e3,
        //     material::lambertian {
        //         albedo: Vec3::new(0.1, 0.1, 0.1),
        //     },
        // ), //Frnt
        sphere(
            Vec3::new(50.0, 1e3, 81.6),
            1e3,
            material::lambertian(constant(Vec3::new(0.75, 0.75, 0.75))),
        ), //Botm
        sphere(
            Vec3::new(50.0, -1e3 + 81.6, 81.6),
            1e3,
            material::lambertian(constant(Vec3::new(0.75, 0.75, 0.75))),
        ), //Top
        sphere(
            Vec3::new(27.0, 16.5, 47.0),
            16.5,
            material::metal(Vec3::new(1.0, 1.0, 1.0) * 0.999, 0.0),
        ), //Mirr
        sphere(Vec3::new(73.0, 16.5, 78.0), 16.5, material::dielectric(1.5)), //Glas
        sphere(
            Vec3::new(50.0, 81.6 - 16.5, 81.6),
            1.5,
            material::diffuse_light(constant(Vec3::new(4.0, 4.0, 4.0) * 100.0)),
        ), //Lite
    ];

    (hitables, camera, Some(Vec3::zero()))
}
