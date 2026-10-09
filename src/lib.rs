//! A Rust implementation of Peter Shirley's "Ray Tracing in One Weekend".
//!
//! The binary (`src/main.rs`) is a thin command line front end. This library target exists so
//! that the criterion benchmarks in `benches/` can build the same scenes, cameras and rays
//! through the same API as the tracer itself.

#![cfg_attr(feature = "core_intrinsics", feature(core_intrinsics))] // for cttz

pub mod camera;
pub mod collision;
pub mod material;
pub mod math;
pub mod offline;
pub mod params;
pub mod perlin;
pub mod pixels_window;
pub mod presets;
pub mod scene;
pub mod simd;
pub mod storage;
pub mod texture;
