use crate::{
    collision::{Hitable, Ray},
    params::Params,
    presets,
    storage::Storage,
};

pub const PARAMS: Params = Params {
    width: 200,
    height: 100,
    samples: 10,
    max_depth: 10,
    random_seed: false,
    use_bvh: false,
};

pub fn hitables_bench<F>(f: F)
where
    F: FnOnce(&Ray, Vec<Hitable>),
{
    let mut rng = PARAMS.new_rng();
    let storage = Storage::new(&mut rng);
    let (hitables, camera, _) = presets::random_spheres(&PARAMS, &mut rng, &storage);
    let ray = camera.get_ray(0.5, 0.5, &mut rng);
    f(&ray, hitables)
}
