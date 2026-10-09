//! Criterion benchmarks for the `SpheresSoA` ray hit paths.
//!
//! `TargetFeature::detect()` decides which paths are runnable, matching the guards in the old
//! nightly benches: `sse4_1` needs any SIMD level, `avx2` needs the AVX2 level specifically.

mod support;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use pathtrace_rs::{
    collision::SpheresSoA,
    presets,
    scene::{MAX_T, MIN_T},
    simd::TargetFeature,
    storage::Storage,
};

fn spheres_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("spheres_soa");

    let mut rng = support::PARAMS.new_rng();
    let storage = Storage::new(&mut rng);
    let (hitables, camera, _) = presets::random_spheres(&support::PARAMS, &mut rng, &storage);
    let ray = camera.get_ray(0.5, 0.5, &mut rng);
    let spheres = SpheresSoA::new(&hitables);

    group.bench_function("scalar", |b| {
        b.iter(|| black_box(spheres.hit_scalar(&ray, MIN_T, MAX_T)))
    });

    let feature = TargetFeature::detect();
    if feature != TargetFeature::FallBack {
        group.bench_function("sse4_1", |b| {
            b.iter(|| unsafe { black_box(spheres.hit_sse4_1(&ray, MIN_T, MAX_T)) })
        });
    }
    if feature == TargetFeature::AVX2 {
        group.bench_function("avx2", |b| {
            b.iter(|| unsafe { black_box(spheres.hit_avx2(&ray, MIN_T, MAX_T)) })
        });
    }

    #[cfg(feature = "fearless_simd")]
    {
        group.bench_function("auto_vectorize", |b| {
            b.iter(|| black_box(spheres.hit_auto_vectorize(&ray, MIN_T, MAX_T)))
        });
        group.bench_function("portable_simd", |b| {
            b.iter(|| black_box(spheres.hit_portable_simd(&ray, MIN_T, MAX_T)))
        });
    }

    group.finish();
}

criterion_group!(benches, spheres_benches);
criterion_main!(benches);
