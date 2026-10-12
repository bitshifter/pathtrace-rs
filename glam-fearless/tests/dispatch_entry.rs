//! `dispatch_entry!` must agree with `dispatch!` for every level the machine offers.

use glam_fearless::{Level, Vec3xN, dispatch, dispatch_entry, prelude::*};

/// Kernel body: per-lane length of three SoA component slices.
#[inline(always)]
fn length_body<S: Simd>(simd: S, a: [&[f32]; 3], out: &mut [f32]) {
    let n = Vec3xN::<S>::LANES;
    let ins = a[0]
        .chunks_exact(n)
        .zip(a[1].chunks_exact(n))
        .zip(a[2].chunks_exact(n));
    for (o, ((x, y), z)) in out.chunks_mut(n).zip(ins) {
        Vec3xN::from_slice(simd, x, y, z).length().store_slice(o);
    }
}

dispatch_entry! {
    /// Per-lane length of three SoA component slices.
    pub struct LengthEntry(level: Level) -> fn(a: [&[f32]; 3], out: &mut [f32]) = length_body;
}

/// Every level the machine can offer, as the conformance tests enumerate them.
fn levels() -> Vec<Level> {
    #[allow(unused_mut)]
    let mut levels = vec![Level::new(), Level::baseline()];
    #[cfg(feature = "force_support_fallback")]
    levels.push(Level::fallback());
    levels
}

/// Deterministic data, kept away from the origin so lengths stay conditioned.
fn soa(n: usize) -> [Vec<f32>; 3] {
    let mut out = [vec![0.0f32; n], vec![0.0f32; n], vec![0.0f32; n]];
    let [xs, ys, zs] = &mut out;
    for (i, ((x, y), z)) in xs
        .iter_mut()
        .zip(ys.iter_mut())
        .zip(zs.iter_mut())
        .enumerate()
    {
        let v = glam::Vec3::new(
            (i as f32 * 0.37).sin() * 2.0 + 1.0,
            (i as f32 * 0.71).cos() * 2.0,
            (i as f32 * 1.13).sin() * 2.0 - 0.5,
        );
        *x = v.x;
        *y = v.y;
        *z = v.z;
    }
    out
}

#[test]
fn entry_matches_dispatch_at_every_level() {
    const N: usize = 64;
    let a = soa(N);
    let refs = [a[0].as_slice(), a[1].as_slice(), a[2].as_slice()];

    let mut expected = vec![0.0f32; N];
    dispatch!(Level::new(), simd => length_body(simd, refs, &mut expected));

    for level in levels() {
        let entry = LengthEntry::with_level(level);
        let mut actual = vec![0.0f32; N];
        entry.call(refs, &mut actual);
        assert_eq!(actual, expected, "{level:?}");

        // The escape hatch must select the same kernel.
        let mut via_fn = vec![0.0f32; N];
        // SAFETY: the pointer comes from an entry built for this level.
        unsafe { entry.as_fn()(refs, &mut via_fn) };
        assert_eq!(via_fn, expected, "{level:?}");
    }
}

#[test]
fn entry_values_are_the_real_lengths() {
    const N: usize = 64;
    let a = soa(N);
    let refs = [a[0].as_slice(), a[1].as_slice(), a[2].as_slice()];
    let mut out = vec![0.0f32; N];

    LengthEntry::new().call(refs, &mut out);

    for i in 0..N {
        let want = (a[0][i] * a[0][i] + a[1][i] * a[1][i] + a[2][i] * a[2][i]).sqrt();
        assert!(
            (out[i] - want).abs() < 1e-5,
            "lane {i}: {} vs {want}",
            out[i]
        );
    }
}

#[test]
fn default_entry_detects_the_level() {
    const N: usize = 32;
    let a = soa(N);
    let refs = [a[0].as_slice(), a[1].as_slice(), a[2].as_slice()];
    let mut out = vec![0.0f32; N];

    LengthEntry::default().call(refs, &mut out);

    assert!(out.iter().all(|v| v.is_finite() && *v > 0.0));
}
