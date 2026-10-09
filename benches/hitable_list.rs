//! Criterion benchmarks for the flat hitable list ray hit path.

mod support;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use pathtrace_rs::{
    collision::HitableList,
    presets,
    scene::{MAX_T, MIN_T},
    storage::Storage,
};

fn hitable_list_benches(c: &mut Criterion) {
    let mut rng = support::PARAMS.new_rng();
    let storage = Storage::new(&mut rng);
    let (hitables, camera, _) = presets::random_spheres(&support::PARAMS, &mut rng, &storage);
    let ray = camera.get_ray(0.5, 0.5, &mut rng);
    let list = HitableList::new(hitables);

    c.bench_function("hitable_list_ray_hit", |b| {
        b.iter(|| black_box(list.ray_hit(&ray, MIN_T, MAX_T, &mut rng)))
    });
}

criterion_group!(benches, hitable_list_benches);
criterion_main!(benches);
