//! What does per-call runtime dispatch cost?
//!
//! `glam_fearless` picks its SIMD width at runtime: `Level::new()` detects it once and caches
//! it, and `dispatch!` selects the matching kernel. Detection is cheap, but it is not free, and
//! it sits in front of every kernel call. A caller that applies a kernel once per batch should
//! detect the level once and pass it down, rather than calling `Level::new()` inside the loop.
//! This bench measures that difference on one small kernel, and nothing else.
//!
//! How to read it: the difference is a fixed per-call cost, so it is a visible fraction of the
//! `16` case (two 8-lane chunks on AVX2) and inside the noise by `1024` (128 chunks). If the gap
//! reads as a small constant number of nanoseconds rather than a slope in ns/element, hoisting
//! the level is the whole story. Compare the two variants within a single run: the absolute
//! 16-element numbers move by several percent between runs on a noisy machine, the delta does not.
//!
//! What this does not isolate: the generated `#[simd]` entry point, which is most of what a
//! fearless call costs over a plain `#[target_feature]` loop. With the level hoisted, this kernel
//! at 16 elements still costs ~5.3 ns against ~2.8 ns for a hand-written AVX2 loop with the same
//! loads, maths and store, and passing a concrete token instead of going through `dispatch!`
//! saves only ~0.3 ns of that: the rest is the generated entry point. SoA slicing is not the cost
//! either, `length` over three component slices measuring the same as over a sliced-AoS pool.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use glam_fearless::{Level, Vec3xN, dispatch, prelude::*, simd};

/// Per-lane length of an SoA batch: three component slices in, one length slice out.
#[simd]
#[inline(always)]
fn length_impl<S: Simd>(simd: S, a: [&[f32]; 3], out: &mut [f32]) {
    let n = Vec3xN::<S>::LANES;
    let ins = a[0]
        .chunks_exact(n)
        .zip(a[1].chunks_exact(n))
        .zip(a[2].chunks_exact(n));
    for (o, ((x, y), z)) in out.chunks_mut(n).zip(ins) {
        let v = Vec3xN::from_slice(simd, x, y, z);
        v.length().store_slice(o);
    }
}

/// The easy mistake: detect the level on every call.
///
/// `#[inline(never)]` keeps the detection inside the timed region. Inlined, the compiler can
/// lift `Level::new()` out of the benchmark loop and silently erase the difference this bench
/// exists to show.
#[inline(never)]
fn length_per_call_level(a: [&[f32]; 3], out: &mut [f32]) {
    let level = Level::new();
    dispatch!(level, simd => length_impl(simd, a, out))
}

/// The shape to prefer: detect once, pass the level down.
#[inline(never)]
fn length_hoisted_level(level: Level, a: [&[f32]; 3], out: &mut [f32]) {
    dispatch!(level, simd => length_impl(simd, a, out))
}

/// Deterministic pseudo-random SoA components in roughly `[-2, 2]`, so no dependency is needed
/// just to make benchmark inputs.
fn soa(size: usize, seed: u32) -> [Vec<f32>; 3] {
    let mut state = seed | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        (state >> 8) as f32 / 16_777_216.0 * 4.0 - 2.0
    };
    let mut out = [vec![0.0f32; size], vec![0.0f32; size], vec![0.0f32; size]];
    for i in 0..size {
        for component in &mut out {
            component[i] = next();
        }
    }
    out
}

fn bench_dispatch_overhead(c: &mut Criterion) {
    /// A kernel of two chunks (16 elements) shows the fixed cost; 128 chunks (1024) hide it.
    const SIZES: &[usize] = &[16, 1024];

    // Detected once, outside the timed loops, the way a caller with a batch to process would.
    let level = Level::new();

    let mut group = c.benchmark_group("dispatch overhead (vector3 length)");
    for &size in SIZES {
        let a = soa(size, 0x9e37_79b9);
        let mut out = vec![0.0f32; size];
        let refs = [a[0].as_slice(), a[1].as_slice(), a[2].as_slice()];

        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new("per_call_level", size), |b| {
            b.iter(|| {
                length_per_call_level(refs, &mut out);
                std::hint::black_box(&out);
            })
        });
        group.bench_function(BenchmarkId::new("hoisted_level", size), |b| {
            b.iter(|| {
                length_hoisted_level(level, refs, &mut out);
                std::hint::black_box(&out);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_dispatch_overhead);
criterion_main!(benches);
