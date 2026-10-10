# glam-fearless (prototype)

Wide ("vertical") SIMD maths types for the glam ecosystem, built on
[`fearless_simd`](https://docs.rs/fearless_simd): `Vec2xN`/`Vec3xN`/`Vec4xN<S>` and mask types
generic over the SIMD token, so one type serves every dispatch level (4/8/16 lanes).

This is a standalone crate (its own `[workspace]`), **not** part of `glam` and not part of the
pathtrace-rs package graph. `pathtrace-rs` depends on it by path and uses it for the wide
`SphereSoA` ray/sphere path (`src/collision/spheres_soa.rs`, feature `glam_fearless`).

Design notes are deliberately kept out of this repository.

## Usage

```sh
cargo test                 # 26 conformance tests against scalar glam, 4- and 8-lane widths
cargo test --features force_support_fallback   # adds the 4-lane emulated scalar backend
cargo run --example smoke
```

Inside pathtrace-rs:

```sh
cargo test
PATHTRACE_SIMD=glam_fearless cargo run --release -- -O -W 160 -H 90 -S 2
cargo bench --bench spheres_soa    # criterion; see the spheres_soa/glam_fearless benchmark
```

## Notes

- Every wide value carries its zero-sized SIMD token, so operators need no explicit token argument:
  `let oc = ro - centre; let b = oc.dot(rd);`
- Ask the width with `Vec3xN::<S>::LANES` inside a `#[simd]` function, or `lane_len_f32(level)` when
  you only have a `Level`.
- The scalar fallback in fearless_simd is a 4-lane *emulated* backend, so lane counts are 4, 8 or 16;
  there is no 1-lane level.
- No `unsafe` in this crate.
