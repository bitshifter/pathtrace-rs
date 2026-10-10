//! Conformance tests: every wide operation must agree with scalar `glam` per lane.
#![allow(clippy::too_many_arguments)]
//!
//! Every test runs for each level the machine can offer — `Level::new()` (best
//! available), `Level::baseline()` (statically supported), and, with the
//! `force_support_fallback` feature, `Level::fallback()` — so the same source is
//! exercised against more than one backend. Note that fearless_simd's scalar
//! fallback is a 4-lane *emulated* backend (`type f32s = f32x4<Fallback>`), not a
//! 1-lane one, so it adds backend coverage rather than a new width.

use glam::{Vec2 as GlamVec2, Vec3 as GlamVec3, Vec4 as GlamVec4};
use glam_fearless::{Level, Vec2xN, Vec3xN, Vec4xN, dispatch, lane_len_f32, prelude::*, simd};

/// Lanes processed by each test; a multiple of every lane count under test.
const N: usize = 64;

fn levels() -> Vec<Level> {
    #[allow(unused_mut)]
    let mut levels = vec![Level::new(), Level::baseline()];
    #[cfg(feature = "force_support_fallback")]
    levels.push(Level::fallback());
    levels
}

/// Deterministic test data in roughly `[-2, 2]`, with `|v| >= 0.6` so divisions and
/// normalizations stay well conditioned.
fn signed(n: usize, seed: f32) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let v = (i as f32 * 0.37 + seed * 1.7).sin() * 2.0;
            if v.abs() < 0.6 { v + 1.5 } else { v }
        })
        .collect()
}

#[track_caller]
fn assert_lanes_eq(actual: &[f32], expected: &[f32], what: &str) {
    assert_eq!(actual.len(), expected.len(), "{what}: length");
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert!(a == e, "{what}: lane {i}: wide={a} scalar={e}");
    }
}

#[track_caller]
fn assert_lanes_close(actual: &[f32], expected: &[f32], eps: f32, what: &str) {
    assert_eq!(actual.len(), expected.len(), "{what}: length");
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        let tol = eps * e.abs().max(1.0);
        assert!(
            (a - e).abs() <= tol,
            "{what}: lane {i}: wide={a} scalar={e} (tol {tol})"
        );
    }
}

/// Compares lane bitmasks (one bit per lane) element by element.
#[track_caller]
fn assert_lane_bits_eq(actual: &[u64], expected: &[u64], what: &str) {
    assert_eq!(actual.len(), expected.len(), "{what}: length");
    for (k, (a, e)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(a, e, "{what}: chunk {k}: {a:#x} vs {e:#x}");
    }
}

/// Generates the SIMD-generic test helpers and the conformance tests for one wide
/// vector type. The component list is `(field, input_a, input_b, output, seed)`.
macro_rules! conformance_type {
    (
        $mod_name:ident, $vec:ident, $glam:ident,
        [$(($f:ident, $a:ident, $b:ident, $o:ident, $seed:expr)),+]
    ) => {
        mod $mod_name {
            use super::*;

            /// `op`: 0 add, 1 sub, 2 mul, 3 div, 4 min, 5 max, 6 neg, 7 abs.
            #[simd]
            #[inline(always)]
            pub fn bin<S: Simd>(
                simd: S,
                op: u32,
                $($a: &[f32], $b: &[f32], $o: &mut [f32],)+
            ) {
                let n = S::f32s::LEN;
                for i in (0..N).step_by(n) {
                    let va = $vec::from_slice(simd, $(&$a[i..i + n]),+);
                    let vb = $vec::from_slice(simd, $(&$b[i..i + n]),+);
                    let r = match op {
                        0 => va + vb,
                        1 => va - vb,
                        2 => va * vb,
                        3 => va / vb,
                        4 => va.min(vb),
                        5 => va.max(vb),
                        6 => -va,
                        _ => va.abs(),
                    };
                    $($o[i..i + n].copy_from_slice(r.$f.as_slice());)+
                }
            }

            /// Scalar broadcast. `op`: 0 `v * s`, 1 `v / s`, 2 `s * v`.
            #[simd]
            #[inline(always)]
            pub fn scalar<S: Simd>(simd: S, op: u32, s: f32, $($a: &[f32], $o: &mut [f32],)+) {
                let n = S::f32s::LEN;
                for i in (0..N).step_by(n) {
                    let va = $vec::from_slice(simd, $(&$a[i..i + n]),+);
                    let r = match op {
                        0 => va * s,
                        1 => va / s,
                        _ => s * va,
                    };
                    $($o[i..i + n].copy_from_slice(r.$f.as_slice());)+
                }
            }

            /// Per-lane horizontal ops. `op`: 0 dot, 1 length_squared, 2 length.
            #[simd]
            #[inline(always)]
            pub fn horiz<S: Simd>(simd: S, op: u32, $($a: &[f32], $b: &[f32],)+ out: &mut [f32]) {
                let n = S::f32s::LEN;
                for i in (0..N).step_by(n) {
                    let va = $vec::from_slice(simd, $(&$a[i..i + n]),+);
                    let vb = $vec::from_slice(simd, $(&$b[i..i + n]),+);
                    let r = match op {
                        0 => va.dot(vb),
                        1 => va.length_squared(),
                        _ => va.length(),
                    };
                    out[i..i + n].copy_from_slice(r.as_slice());
                }
            }

            /// Per-lane normalize.
            #[simd]
            #[inline(always)]
            pub fn normalize<S: Simd>(simd: S, $($a: &[f32], $o: &mut [f32],)+) {
                let n = S::f32s::LEN;
                for i in (0..N).step_by(n) {
                    let va = $vec::new(simd, $(S::f32s::from_slice(simd, &$a[i..i + n])),+);
                    let r = va.normalize();
                    $($o[i..i + n].copy_from_slice(r.$f.as_slice());)+
                }
            }

            /// Component-wise comparisons, reported as one lane bitmask per component.
            /// `op`: 0 gt, 1 ge, 2 lt, 3 le, 4 eq, 5 ne.
            #[simd]
            #[inline(always)]
            pub fn cmp<S: Simd>(simd: S, op: u32, $($a: &[f32], $b: &[f32], $o: &mut [u64],)+) {
                let n = S::f32s::LEN;
                for i in (0..N).step_by(n) {
                    let va = $vec::from_slice(simd, $(&$a[i..i + n]),+);
                    let vb = $vec::from_slice(simd, $(&$b[i..i + n]),+);
                    let m = match op {
                        0 => va.cmp_gt(vb),
                        1 => va.cmp_ge(vb),
                        2 => va.cmp_lt(vb),
                        3 => va.cmp_le(vb),
                        4 => va.cmp_eq(vb),
                        _ => va.cmp_ne(vb),
                    };
                    $($o[i / n] = m.$f.to_bitmask();)+
                }
            }

            /// Component-wise mask logic and `any_true`, as lane bitmasks.
            ///
            /// Per chunk and component the output holds `[and, or, not, any_true]`.
            #[simd]
            #[inline(always)]
            pub fn mask_ops<S: Simd>(simd: S, $($a: &[f32], $b: &[f32], $o: &mut [u64],)+) {
                let n = S::f32s::LEN;
                for i in (0..N).step_by(n) {
                    let va = $vec::from_slice(simd, $(&$a[i..i + n]),+);
                    let vb = $vec::from_slice(simd, $(&$b[i..i + n]),+);
                    let gt = va.cmp_gt(vb);
                    let lt = va.cmp_lt(vb);
                    // Exercise the crate's own BVec operators, not the raw lane masks.
                    let both = gt & lt;
                    let either = gt | lt;
                    let negated = !gt;
                    let k = i / n;
                    $(
                        $o[4 * k] = both.$f.to_bitmask();
                        $o[4 * k + 1] = either.$f.to_bitmask();
                        $o[4 * k + 2] = negated.$f.to_bitmask();
                        $o[4 * k + 3] = u64::from(gt.$f.any_true());
                    )+
                }
            }

            /// `Vec3xN::select`, the static form glam uses (`Vec3::select(mask, a, b)`).
            #[simd]
            #[inline(always)]
            pub fn select<S: Simd>(simd: S, $($a: &[f32], $b: &[f32], $o: &mut [f32],)+) {
                let n = S::f32s::LEN;
                for i in (0..N).step_by(n) {
                    let va = $vec::from_slice(simd, $(&$a[i..i + n]),+);
                    let vb = $vec::from_slice(simd, $(&$b[i..i + n]),+);
                    let r = $vec::select(va.cmp_gt(vb), va, vb);
                    $($o[i..i + n].copy_from_slice(r.$f.as_slice());)+
                }
            }

            #[test]
            fn binary_ops_match_glam() {
                let label = ["add", "sub", "mul", "div", "min", "max", "neg", "abs"];
                for level in levels() {
                    $(let $a = signed(N, $seed);)+
                    $(let $b = signed(N, $seed + 0.5);)+
                    let ga: Vec<$glam> = (0..N).map(|i| $glam::new($($a[i],)+)).collect();
                    let gb: Vec<$glam> = (0..N).map(|i| $glam::new($($b[i],)+)).collect();
                    for op in 0..8u32 {
                        $(let mut $o = vec![0.0f32; N];)+
                        dispatch!(level, simd => bin(simd, op, $(&$a, &$b, &mut $o),+));
                        let what = format!("{} level={level:?}", label[op as usize]);
                        $(
                            let exp: Vec<f32> = ga.iter().zip(&gb).map(|(x, y)| match op {
                                0 => (x + y).$f,
                                1 => (x - y).$f,
                                2 => (x * y).$f,
                                3 => (x / y).$f,
                                4 => x.min(*y).$f,
                                5 => x.max(*y).$f,
                                6 => (-x).$f,
                                _ => x.abs().$f,
                            }).collect();
                            assert_lanes_eq(&$o, &exp, &what);
                        )+
                    }
                }
            }

            #[test]
            fn scalar_ops_match_glam() {
                let label = ["mul_scalar", "div_scalar", "scalar_mul"];
                let s = 2.5f32;
                for level in levels() {
                    $(let $a = signed(N, $seed);)+
                    let ga: Vec<$glam> = (0..N).map(|i| $glam::new($($a[i],)+)).collect();
                    for op in 0..3u32 {
                        $(let mut $o = vec![0.0f32; N];)+
                        dispatch!(level, simd => scalar(simd, op, s, $(&$a, &mut $o),+));
                        let what = format!("{} level={level:?}", label[op as usize]);
                        $(
                            let exp: Vec<f32> = ga.iter().map(|x| match op {
                                0 => (x * s).$f,
                                1 => (x / s).$f,
                                _ => (s * x).$f,
                            }).collect();
                            assert_lanes_eq(&$o, &exp, &what);
                        )+
                    }
                }
            }

            #[test]
            fn horizontal_ops_match_glam() {
                let label = ["dot", "length_squared", "length"];
                for level in levels() {
                    $(let $a = signed(N, $seed);)+
                    $(let $b = signed(N, $seed + 0.5);)+
                    let ga: Vec<$glam> = (0..N).map(|i| $glam::new($($a[i],)+)).collect();
                    let gb: Vec<$glam> = (0..N).map(|i| $glam::new($($b[i],)+)).collect();
                    for op in 0..3u32 {
                        let mut out = vec![0.0f32; N];
                        dispatch!(level, simd => horiz(simd, op, $(&$a, &$b,)+ &mut out));
                        let exp: Vec<f32> = ga.iter().zip(&gb).map(|(x, y)| match op {
                            0 => x.dot(*y),
                            1 => x.length_squared(),
                            _ => x.length(),
                        }).collect();
                        let what = format!("{} level={level:?}", label[op as usize]);
                        // `dot`/`length` may be contracted to FMA on some levels.
                        assert_lanes_close(&out, &exp, 1e-6, &what);
                    }
                }
            }

            #[test]
            fn normalize_matches_glam() {
                for level in levels() {
                    $(let $a = signed(N, $seed);)+
                    let ga: Vec<$glam> = (0..N).map(|i| $glam::new($($a[i],)+)).collect();
                    $(let mut $o = vec![0.0f32; N];)+
                    dispatch!(level, simd => normalize(simd, $(&$a, &mut $o),+));
                    $(
                        let exp: Vec<f32> = ga.iter().map(|v| v.normalize().$f).collect();
                        let what = format!("normalize level={level:?} component={}", stringify!($f));
                        // glam's scalar normalize multiplies by the reciprocal; ours divides
                        // (as glam's SIMD path does), so the results differ by rounding only.
                        assert_lanes_close(&$o, &exp, 1e-6, &what);
                    )+
                }
            }

            #[test]
            fn comparisons_match_glam() {
                for level in levels() {
                    $(let $a = signed(N, $seed);)+
                    let ga: Vec<$glam> = (0..N).map(|i| $glam::new($($a[i],)+)).collect();
                    let n = lane_len_f32(level);
                    let chunks = N / n;
                    // `lt`/`le` and `gt`/`ge` only differ on equal operands, so run the
                    // sweep twice: distinct values, then `b == a` everywhere.
                    for equal in [false, true] {
                    $(let $b = if equal { $a.clone() } else { signed(N, $seed + 0.5) };)+
                    let gb: Vec<$glam> = if equal {
                        ga.clone()
                    } else {
                        (0..N).map(|i| $glam::new($($b[i],)+)).collect()
                    };
                    for op in 0..6u32 {
                        $(let mut $o = vec![0u64; chunks];)+
                        dispatch!(level, simd => cmp(simd, op, $(&$a, &$b, &mut $o),+));
                        $(
                            let exp: Vec<u64> = (0..chunks)
                                .map(|k| {
                                    let mut bits = 0u64;
                                    for j in 0..n {
                                        let x = ga[k * n + j];
                                        let y = gb[k * n + j];
                                        // Scalar comparison on the component: glam's
                                        // `Vec4xN` comparisons return `BVec4A`, which has no
                                        // per-component fields to read.
                                        let b = match op {
                                            0 => x.$f > y.$f,
                                            1 => x.$f >= y.$f,
                                            2 => x.$f < y.$f,
                                            3 => x.$f <= y.$f,
                                            4 => x.$f == y.$f,
                                            _ => x.$f != y.$f,
                                        };
                                        if b {
                                            bits |= 1u64 << j;
                                        }
                                    }
                                    bits
                                })
                                .collect();
                            let what = format!(
                                "cmp op={op} equal={equal} level={level:?} component={}",
                                stringify!($f)
                            );
                            assert_lane_bits_eq(&$o, &exp, &what);
                        )+
                    }
                    }
                }
            }

            #[test]
            fn mask_logic_matches_glam() {
                for level in levels() {
                    $(let $a = signed(N, $seed);)+
                    $(let $b = signed(N, $seed + 0.5);)+
                    let ga: Vec<$glam> = (0..N).map(|i| $glam::new($($a[i],)+)).collect();
                    let gb: Vec<$glam> = (0..N).map(|i| $glam::new($($b[i],)+)).collect();
                    let n = lane_len_f32(level);
                    let chunks = N / n;
                    $(let mut $o = vec![0u64; 4 * chunks];)+
                    dispatch!(level, simd => mask_ops(simd, $(&$a, &$b, &mut $o),+));
                    $(
                        let mut exp = vec![0u64; 4 * chunks];
                        for k in 0..chunks {
                            for j in 0..n {
                                let x = ga[k * n + j];
                                let y = gb[k * n + j];
                                let gt = x.$f > y.$f;
                                let lt = x.$f < y.$f;
                                if gt && lt {
                                    exp[4 * k] |= 1u64 << j;
                                }
                                if gt || lt {
                                    exp[4 * k + 1] |= 1u64 << j;
                                }
                                if !gt {
                                    exp[4 * k + 2] |= 1u64 << j;
                                }
                                if gt {
                                    exp[4 * k + 3] = 1;
                                }
                            }
                        }
                        let what = format!(
                            "mask ops level={level:?} component={}",
                            stringify!($f)
                        );
                        assert_lane_bits_eq(&$o, &exp, &what);
                    )+
                }
            }

            #[test]
            fn select_matches_glam() {
                for level in levels() {
                    $(let $a = signed(N, $seed);)+
                    $(let $b = signed(N, $seed + 0.5);)+
                    let ga: Vec<$glam> = (0..N).map(|i| $glam::new($($a[i],)+)).collect();
                    let gb: Vec<$glam> = (0..N).map(|i| $glam::new($($b[i],)+)).collect();
                    $(let mut $o = vec![0.0f32; N];)+
                    dispatch!(level, simd => select(simd, $(&$a, &$b, &mut $o),+));
                    $(
                        let exp: Vec<f32> = ga
                            .iter()
                            .zip(&gb)
                            .map(|(x, y)| $glam::select(x.cmpgt(*y), *x, *y).$f)
                            .collect();
                        let c = stringify!($f);
                        assert_lanes_eq(&$o, &exp, &format!("select {c} level={level:?}"));
                    )+
                }
            }
        }
    };
}

conformance_type!(
    vec2,
    Vec2xN,
    GlamVec2,
    [(x, ax, bx, ox, 1.0), (y, ay, by, oy, 2.5)]
);

conformance_type!(
    vec3,
    Vec3xN,
    GlamVec3,
    [
        (x, ax, bx, ox, 1.0),
        (y, ay, by, oy, 2.5),
        (z, az, bz, oz, 4.25)
    ]
);

conformance_type!(
    vec4,
    Vec4xN,
    GlamVec4,
    [
        (x, ax, bx, ox, 1.0),
        (y, ay, by, oy, 2.5),
        (z, az, bz, oz, 4.25),
        (w, aw, bw, ow, 7.5)
    ]
);

// --- Constructors, `SimdInto`, and compound assignment. ---

#[simd]
#[inline(always)]
fn constructors_impl<S: Simd>(simd: S, s: f32, ox: &mut [f32], oy: &mut [f32], oz: &mut [f32]) {
    let n = S::f32s::LEN;
    let mut acc = Vec3xN::splat(simd, s);
    let zero = Vec3xN::zero(simd);
    let mixed = Vec3xN::splat_xyz(simd, s, s + 1.0, s + 2.0);
    let from_scalar: Vec3xN<S> = s.simd_into(simd);
    let from_glam_vec = Vec3xN::splat_vec(simd, glam::Vec3::new(s, s + 1.0, s + 2.0));
    let from_array = Vec3xN::splat_vec(simd, [s, s + 1.0, s + 2.0]);
    // x2 then /2 is exact for any normal f32, so `acc` is `s` again afterwards.
    acc += acc;
    acc -= Vec3xN::splat(simd, s);
    acc *= Vec3xN::splat(simd, 2.0);
    acc /= Vec3xN::splat(simd, 2.0);
    let rows = [acc, zero, mixed, from_scalar, from_glam_vec, from_array];
    for (r, v) in rows.iter().enumerate() {
        let off = r * n;
        v.x.store_slice(&mut ox[off..off + n]);
        v.y.store_slice(&mut oy[off..off + n]);
        v.z.store_slice(&mut oz[off..off + n]);
    }
}

#[test]
fn constructors_and_assign_match_glam() {
    let s = 2.5f32;
    for level in levels() {
        let n = lane_len_f32(level);
        let (mut ox, mut oy, mut oz) = (
            vec![0.0f32; 6 * n],
            vec![0.0f32; 6 * n],
            vec![0.0f32; 6 * n],
        );
        dispatch!(level, simd => constructors_impl(simd, s, &mut ox, &mut oy, &mut oz));
        let expected = [
            glam::Vec3::splat(s),
            glam::Vec3::ZERO,
            glam::Vec3::new(s, s + 1.0, s + 2.0),
            glam::Vec3::splat(s),
            // `splat_vec` from a glam vector and from an array.
            glam::Vec3::new(s, s + 1.0, s + 2.0),
            glam::Vec3::new(s, s + 1.0, s + 2.0),
        ];
        for (r, e) in expected.iter().enumerate() {
            let off = r * n;
            let what = format!("constructors row={r} level={level:?}");
            assert_lanes_eq(&ox[off..off + n], &vec![e.x; n], &what);
            assert_lanes_eq(&oy[off..off + n], &vec![e.y; n], &what);
            assert_lanes_eq(&oz[off..off + n], &vec![e.z; n], &what);
        }
    }
}

// --- Type-specific ops. ---

#[simd]
#[inline(always)]
fn vec2_perp_dot<S: Simd>(
    simd: S,
    ax: &[f32],
    ay: &[f32],
    bx: &[f32],
    by: &[f32],
    out: &mut [f32],
) {
    let n = S::f32s::LEN;
    for i in (0..N).step_by(n) {
        let a = Vec2xN::from_slice(simd, &ax[i..i + n], &ay[i..i + n]);
        let b = Vec2xN::from_slice(simd, &bx[i..i + n], &by[i..i + n]);
        out[i..i + n].copy_from_slice(a.perp_dot(b).as_slice());
    }
}

#[simd]
#[inline(always)]
fn vec3_cross<S: Simd>(
    simd: S,
    ax: &[f32],
    ay: &[f32],
    az: &[f32],
    bx: &[f32],
    by: &[f32],
    bz: &[f32],
    ox: &mut [f32],
    oy: &mut [f32],
    oz: &mut [f32],
) {
    let n = S::f32s::LEN;
    for i in (0..N).step_by(n) {
        let a = Vec3xN::from_slice(simd, &ax[i..i + n], &ay[i..i + n], &az[i..i + n]);
        let b = Vec3xN::from_slice(simd, &bx[i..i + n], &by[i..i + n], &bz[i..i + n]);
        let r = a.cross(b);
        ox[i..i + n].copy_from_slice(r.x.as_slice());
        oy[i..i + n].copy_from_slice(r.y.as_slice());
        oz[i..i + n].copy_from_slice(r.z.as_slice());
    }
}

#[simd]
#[inline(always)]
fn vec4_truncate<S: Simd>(
    simd: S,
    ax: &[f32],
    ay: &[f32],
    az: &[f32],
    aw: &[f32],
    ox: &mut [f32],
    oy: &mut [f32],
    oz: &mut [f32],
) {
    let n = S::f32s::LEN;
    for i in (0..N).step_by(n) {
        let a = Vec4xN::from_slice(
            simd,
            &ax[i..i + n],
            &ay[i..i + n],
            &az[i..i + n],
            &aw[i..i + n],
        );
        let r = a.truncate();
        ox[i..i + n].copy_from_slice(r.x.as_slice());
        oy[i..i + n].copy_from_slice(r.y.as_slice());
        oz[i..i + n].copy_from_slice(r.z.as_slice());
    }
}

#[test]
fn perp_dot_matches_glam() {
    for level in levels() {
        let (ax, ay) = (signed(N, 1.0), signed(N, 2.5));
        let (bx, by) = (signed(N, 4.0), signed(N, 5.5));
        let mut out = vec![0.0f32; N];
        dispatch!(level, simd => vec2_perp_dot(simd, &ax, &ay, &bx, &by, &mut out));
        let exp: Vec<f32> = (0..N)
            .map(|i| glam::Vec2::new(ax[i], ay[i]).perp_dot(glam::Vec2::new(bx[i], by[i])))
            .collect();
        assert_lanes_eq(&out, &exp, &format!("perp_dot level={level:?}"));
    }
}

#[test]
fn cross_matches_glam() {
    for level in levels() {
        let (ax, ay, az) = (signed(N, 1.0), signed(N, 2.5), signed(N, 4.25));
        let (bx, by, bz) = (signed(N, 6.0), signed(N, 7.5), signed(N, 9.25));
        let (mut ox, mut oy, mut oz) = (vec![0.0f32; N], vec![0.0f32; N], vec![0.0f32; N]);
        dispatch!(level, simd => vec3_cross(simd, &ax, &ay, &az, &bx, &by, &bz, &mut ox, &mut oy, &mut oz));
        let exp: Vec<[f32; 3]> = (0..N)
            .map(|i| {
                glam::Vec3::new(ax[i], ay[i], az[i])
                    .cross(glam::Vec3::new(bx[i], by[i], bz[i]))
                    .into()
            })
            .collect();
        for (i, e) in exp.iter().enumerate() {
            assert_eq!([ox[i], oy[i], oz[i]], *e, "cross level={level:?} lane {i}");
        }
    }
}

#[test]
fn truncate_matches_glam() {
    for level in levels() {
        let (ax, ay, az, aw) = (
            signed(N, 1.0),
            signed(N, 2.5),
            signed(N, 4.25),
            signed(N, 7.5),
        );
        let (mut ox, mut oy, mut oz) = (vec![0.0f32; N], vec![0.0f32; N], vec![0.0f32; N]);
        dispatch!(level, simd => vec4_truncate(simd, &ax, &ay, &az, &aw, &mut ox, &mut oy, &mut oz));
        for i in 0..N {
            let e = glam::Vec4::new(ax[i], ay[i], az[i], aw[i]).truncate();
            assert_eq!(
                [ox[i], oy[i], oz[i]],
                [e.x, e.y, e.z],
                "truncate level={level:?} lane {i}"
            );
        }
    }
}

// --- Padding / tail lanes. ---

/// Centre components in padding lanes are `f32::MAX`, as `SpheresSoA` does.
const PAD: f32 = f32::MAX;

/// One padded chunk: per-lane squared distance from `origin`, plus the first lane
/// whose distance is below the padding sentinel (the `SpheresSoA` tie-break).
#[simd]
#[inline(always)]
fn padded_min_impl<S: Simd>(
    simd: S,
    origin: [f32; 3],
    xs: &[f32],
    ys: &[f32],
    zs: &[f32],
    out_lane: &mut [u32],
    out_min: &mut [f32],
) {
    let n = S::f32s::LEN;
    let o = Vec3xN::splat_vec(simd, origin);
    let c = Vec3xN::from_slice(simd, &xs[..n], &ys[..n], &zs[..n]);
    let d2 = (o - c).length_squared();
    let valid = d2.simd_lt(S::f32s::splat(simd, PAD));
    let min_t = d2.reduce_min();
    out_min[0] = min_t;
    // Tie-break by lowest lane, like the hand-written paths in `spheres_soa.rs`.
    let min_mask = d2.simd_eq(min_t) & valid;
    out_lane[0] = if min_mask.any_true() {
        min_mask.to_bitmask().trailing_zeros()
    } else {
        u32::MAX
    };
}

#[test]
fn padding_lanes_are_ignored() {
    let origin = [0.5, -0.5, 1.5];
    for level in levels() {
        let n = lane_len_f32(level);
        for valid_count in 1..=n {
            let mut xs = signed(n, 1.0);
            let mut ys = signed(n, 2.5);
            let mut zs = signed(n, 4.25);
            for lane in valid_count..n {
                xs[lane] = PAD;
                ys[lane] = PAD;
                zs[lane] = PAD;
            }
            let mut lane = [u32::MAX; 1];
            let mut min = [f32::NAN; 1];
            dispatch!(level, simd => padded_min_impl(simd, origin, &xs, &ys, &zs, &mut lane, &mut min));

            let d2 = |i: usize| {
                let (dx, dy, dz) = (xs[i] - origin[0], ys[i] - origin[1], zs[i] - origin[2]);
                dx * dx + dy * dy + dz * dz
            };
            let expected_min = (0..valid_count).map(d2).fold(f32::INFINITY, f32::min);
            // First minimum wins on ties, matching the lowest-lane tie-break.
            let mut expected_lane = 0usize;
            for i in 1..valid_count {
                if d2(i) < d2(expected_lane) {
                    expected_lane = i;
                }
            }

            let what = format!("level={level:?} valid={valid_count} lanes={n}");
            assert_eq!(min[0], expected_min, "padded min {what}");
            assert_eq!(lane[0], expected_lane as u32, "padded winning lane {what}");
            assert!((lane[0] as usize) < valid_count, "padding lane won {what}");
        }
    }
}

// --- Per-type constructor names. ---

#[simd]
#[inline(always)]
fn splat_names_impl<S: Simd>(simd: S, x: &mut [f32], y: &mut [f32], z: &mut [f32], w: &mut [f32]) {
    let n = S::f32s::LEN;
    let v2 = Vec2xN::splat_xy(simd, 1.0, 2.0);
    let v3 = Vec3xN::splat_xyz(simd, 1.0, 2.0, 3.0);
    let v4 = Vec4xN::splat_xyzw(simd, 1.0, 2.0, 3.0, 4.0);
    x[..n].copy_from_slice(v2.x.as_slice());
    y[..n].copy_from_slice(v3.z.as_slice());
    z[..n].copy_from_slice(v4.w.as_slice());
    w[..n].copy_from_slice(v4.y.as_slice());
}

/// Each wide type exposes its own per-component splat name.
#[test]
fn per_type_splat_names_work() {
    for level in levels() {
        let n = lane_len_f32(level);
        let (mut x, mut y, mut z, mut w) = (
            vec![0.0f32; n],
            vec![0.0f32; n],
            vec![0.0f32; n],
            vec![0.0f32; n],
        );
        dispatch!(level, simd => splat_names_impl(simd, &mut x, &mut y, &mut z, &mut w));
        assert_eq!(x[0], 1.0, "Vec2xN::splat_xy level={level:?}");
        assert_eq!(y[0], 3.0, "Vec3xN::splat_xyz level={level:?}");
        assert_eq!(z[0], 4.0, "Vec4xN::splat_xyzw level={level:?}");
        assert_eq!(w[0], 2.0, "Vec4xN::splat_xyzw y level={level:?}");
    }
}

// --- Mask encoding round-trips. ---

#[test]
fn levels_cover_multiple_backends() {
    let levels = levels();
    let lens: Vec<usize> = levels.iter().map(|level| lane_len_f32(*level)).collect();
    println!("conformance levels: {lens:?} lanes");
    assert!(
        lens.iter().max() > lens.iter().min(),
        "expected more than one lane count, got {lens:?}"
    );
    #[cfg(feature = "force_support_fallback")]
    assert!(
        levels.iter().any(|level| level.is_fallback()),
        "scalar fallback level missing: {lens:?}"
    );
}

#[simd]
#[inline(always)]
fn mask_round_trip<S: Simd>(simd: S, bits_in: &[u64], bits_out: &mut [u64], any: &mut [u32]) {
    let n = S::f32s::LEN;
    for (k, bits) in bits_in.iter().enumerate() {
        let m = S::mask32s::from_bitmask(simd, *bits);
        bits_out[k] = m.to_bitmask();
        any[k] = u32::from(m.any_true());
        let _ = n;
    }
}

#[test]
fn mask_bitmask_round_trips() {
    for level in levels() {
        let n = lane_len_f32(level);
        let bits: Vec<u64> = vec![0, 1, (1 << n) - 1, if n > 1 { 0b10 } else { 0 }];
        let mut out = vec![0u64; bits.len()];
        let mut any = vec![0u32; bits.len()];
        dispatch!(level, simd => mask_round_trip(simd, &bits, &mut out, &mut any));
        let expected: Vec<u64> = bits.iter().map(|b| b & ((1 << n) - 1)).collect();
        assert_eq!(out, expected, "bitmask round-trip level={level:?}");
        let expected_any: Vec<u32> = expected.iter().map(|b| u32::from(*b != 0)).collect();
        assert_eq!(any, expected_any, "any_true level={level:?}");
    }
}
