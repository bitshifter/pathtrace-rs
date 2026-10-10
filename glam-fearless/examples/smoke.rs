//! Smoke test and usage template: runs a `#[simd]` kernel over the wide types, under the
//! best SIMD level this CPU supports.

use glam_fearless::{Level, Vec3xN, dispatch, prelude::*, simd};

/// Per-lane squared distance from `origin` to each `(x, y, z)` triple, reduced to the
/// minimum over all processed lanes. Leftover lanes are ignored.
#[simd]
#[inline(always)]
fn min_distance_squared_impl<S: Simd>(
    simd: S,
    origin: [f32; 3],
    xs: &[f32],
    ys: &[f32],
    zs: &[f32],
) -> f32 {
    let n = Vec3xN::<S>::LANES;
    let o = Vec3xN::splat_vec(simd, origin);
    let mut best = S::f32s::splat(simd, f32::INFINITY);
    for ((x, y), z) in xs
        .chunks_exact(n)
        .zip(ys.chunks_exact(n))
        .zip(zs.chunks_exact(n))
    {
        let c = Vec3xN::from_slice(simd, x, y, z);
        // No `simd` argument: the operators take the token from their operands.
        let oc = o - c;
        best = best.min(oc.length_squared());
    }
    best.reduce_min()
}

/// Runtime dispatch: one kernel body, compiled once per level and selected here.
fn min_distance_squared(level: Level, origin: [f32; 3], xs: &[f32], ys: &[f32], zs: &[f32]) -> f32 {
    dispatch!(level, simd => min_distance_squared_impl(simd, origin, xs, ys, zs))
}

fn main() {
    let level = Level::new();
    let n = 64usize;
    let xs: Vec<f32> = (0..n).map(|i| i as f32).collect();
    let ys: Vec<f32> = (0..n).map(|i| (i as f32) * 0.5).collect();
    let zs: Vec<f32> = (0..n).map(|i| (i as f32) * 0.25).collect();
    let origin = [8.0, 4.0, 2.0];

    let expected = xs
        .iter()
        .zip(&ys)
        .zip(&zs)
        .map(|((x, y), z)| {
            let (dx, dy, dz) = (x - origin[0], y - origin[1], z - origin[2]);
            dx * dx + dy * dy + dz * dz
        })
        .fold(f32::INFINITY, f32::min);

    let got = min_distance_squared(level, origin, &xs, &ys, &zs);
    println!("level: {level:?}");
    println!("min distance squared: wide={got} scalar={expected}");
    assert_eq!(got, expected);
}
