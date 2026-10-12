//! What does each way of calling a kernel cost?
//!
//! One workload - per-lane length of three SoA component slices - called through every entry this
//! crate offers, so the price of reaching the kernel can be told apart from the price of the kernel
//! itself:
//!
//! * `per_call_level`      - `Level::new()` on every call, `#[simd]` kernel
//! * `dispatch_per_call`   - level detected once, `dispatch!` on every call
//! * `entry_held`          - `dispatch_entry!`, selected once by the caller
//! * `entry_lazy`          - the same entry, selected on first use
//! * `hand_target_feature` - hand-written `#[target_feature(enable = "avx2")]` loop (x86_64 only)
//! * `batch_entry`         - one `dispatch!` for 64 kernel calls, so the selection is paid once
//!
//! Two things are worth knowing when reading the numbers. The difference between the rows is a
//! fixed cost per call, so it is a visible fraction of the 16-element case (two 8-lane chunks) and
//! gone by 1024, where the kernel is throughput bound. And the inputs are cycled through eight
//! buffers: with a single input, a batch loop can hoist its loads, which no caller gets.
//!
//! Run this with and without `-C target-cpu=native`. Without it the caller has no AVX2, which is
//! when runtime dispatch is worth anything; with it an entry calls the kernel directly and inlines
//! it. `src/entry.rs` has the measured results and what they mean.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use glam_fearless::{Level, Vec3xN, dispatch, dispatch_entry, prelude::*, simd};

/// The kernel as a `#[simd]` kernel, for the rows that go through `dispatch!`.
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

/// The same kernel as a plain generic function, which is the form an entry needs.
#[inline(always)]
fn length_body<S: Simd>(simd: S, a: [&[f32]; 3], out: &mut [f32]) {
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

dispatch_entry! {
    /// Per-lane length of three SoA component slices.
    pub struct LengthEntry(level: Level) -> fn(a: [&[f32]; 3], out: &mut [f32]) = length_body;
}

#[inline(never)]
fn per_call_level(a: [&[f32]; 3], out: &mut [f32]) {
    let level = Level::new();
    dispatch!(level, simd => length_impl(simd, a, out))
}

#[inline(never)]
fn dispatch_per_call(level: Level, a: [&[f32]; 3], out: &mut [f32]) {
    dispatch!(level, simd => length_impl(simd, a, out))
}

/// The floor: a hand-written loop with the same loads, maths and store.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
#[inline(never)]
unsafe fn hand_target_feature(a: [&[f32]; 3], out: &mut [f32]) {
    use core::arch::x86_64::*;
    const N: usize = 8;
    let chunks = (out.len() / N)
        .min(a[0].len() / N)
        .min(a[1].len() / N)
        .min(a[2].len() / N);
    let (x, y, z) = (a[0].as_ptr(), a[1].as_ptr(), a[2].as_ptr());
    let o = out.as_mut_ptr();
    for i in 0..chunks {
        let off = i * N;
        let vx = unsafe { _mm256_loadu_ps(x.add(off)) };
        let vy = unsafe { _mm256_loadu_ps(y.add(off)) };
        let vz = unsafe { _mm256_loadu_ps(z.add(off)) };
        let mut acc = _mm256_mul_ps(vx, vx);
        acc = _mm256_add_ps(acc, _mm256_mul_ps(vy, vy));
        acc = _mm256_add_ps(acc, _mm256_mul_ps(vz, vz));
        unsafe { _mm256_storeu_ps(o.add(off), _mm256_sqrt_ps(acc)) };
    }
}

/// Deterministic inputs, in roughly `[-2, 2]`.
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
    /// Kernel calls per timed iteration in the `batch_entry` row.
    const BATCH: usize = 64;
    /// Input buffers to cycle through.
    const POOL: usize = 8;
    const SIZES: &[usize] = &[16, 1024];

    // Detected once, the way a caller with a batch to process would.
    let level = Level::new();

    let mut group = c.benchmark_group("dispatch overhead (vector3 length)");
    for &size in SIZES {
        let pool_soa: Vec<[Vec<f32>; 3]> = (0..POOL)
            .map(|k| soa(size, 0x9e37_79b9u32.wrapping_add(k as u32 * 0x9e37)))
            .collect();
        let pool: Vec<[&[f32]; 3]> = pool_soa
            .iter()
            .map(|s| [s[0].as_slice(), s[1].as_slice(), s[2].as_slice()])
            .collect();
        let mut out = vec![0.0f32; size];

        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(BenchmarkId::new("per_call_level", size), |b| {
            let mut inputs = pool.iter().cycle();
            b.iter(|| {
                per_call_level(*inputs.next().unwrap(), &mut out);
                std::hint::black_box(&out);
            })
        });
        group.bench_function(BenchmarkId::new("dispatch_per_call", size), |b| {
            let mut inputs = pool.iter().cycle();
            b.iter(|| {
                dispatch_per_call(level, *inputs.next().unwrap(), &mut out);
                std::hint::black_box(&out);
            })
        });
        group.bench_function(BenchmarkId::new("entry_held", size), |b| {
            // Selected once, outside the timed loop.
            let kernel = LengthEntry::new();
            let mut inputs = pool.iter().cycle();
            b.iter(|| {
                kernel.call(*inputs.next().unwrap(), &mut out);
                std::hint::black_box(&out);
            })
        });
        group.bench_function(BenchmarkId::new("entry_lazy", size), |b| {
            let mut inputs = pool.iter().cycle();
            b.iter(|| {
                LengthEntry::lazy().call(*inputs.next().unwrap(), &mut out);
                std::hint::black_box(&out);
            })
        });
        #[cfg(target_arch = "x86_64")]
        group.bench_function(BenchmarkId::new("hand_target_feature", size), |b| {
            let mut inputs = pool.iter().cycle();
            b.iter(|| {
                // SAFETY: this host has AVX2, and the row only means something where the caller
                // could use it anyway.
                unsafe { hand_target_feature(*inputs.next().unwrap(), &mut out) };
                std::hint::black_box(&out);
            })
        });

        group.throughput(Throughput::Elements((size * BATCH) as u64));
        group.bench_function(BenchmarkId::new("batch_entry", size), |b| {
            b.iter(|| {
                dispatch!(level, simd => {
                    for input in pool.iter().cycle().take(BATCH) {
                        length_body(simd, *input, &mut out);
                    }
                });
                std::hint::black_box(&out);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_dispatch_overhead);
criterion_main!(benches);
