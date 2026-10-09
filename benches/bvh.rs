//! Criterion benchmarks for the BVH ray hit path.

mod support;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use pathtrace_rs::{
    collision::BVHNode,
    presets,
    scene::{MAX_T, MIN_T},
    storage::Storage,
};

fn bvh_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("bvh");

    // All-sphere scene, which the tracer selects a `SpheresSoA` for.
    {
        let mut rng = support::PARAMS.new_rng();
        let storage = Storage::new(&mut rng);
        let (mut hitables, camera, _) =
            presets::random_spheres(&support::PARAMS, &mut rng, &storage);
        let ray = camera.get_ray(0.5, 0.5, &mut rng);
        let bvh_root = BVHNode::new(&mut rng, &mut hitables, &storage.bvhnode_arena).unwrap();
        group.bench_function("random_spheres_ray_hit", |b| {
            b.iter(|| black_box(bvh_root.ray_hit(&ray, MIN_T, MAX_T, &mut rng)))
        });
    }

    // Mixed scene.
    {
        let mut rng = support::PARAMS.new_rng();
        let storage = Storage::new(&mut rng);
        let (mut hitables, camera, _) = presets::random(&support::PARAMS, &mut rng, &storage);
        let ray = camera.get_ray(0.5, 0.5, &mut rng);
        let bvh_root = BVHNode::new(&mut rng, &mut hitables, &storage.bvhnode_arena).unwrap();
        group.bench_function("ray_hit", |b| {
            b.iter(|| black_box(bvh_root.ray_hit(&ray, MIN_T, MAX_T, &mut rng)))
        });
    }

    group.finish();
}

criterion_group!(benches, bvh_benches);
criterion_main!(benches);
