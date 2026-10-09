//! Shared setup for the criterion benchmarks.

#![allow(dead_code)]

use pathtrace_rs::params::{Params, SoaMode};

/// Scene parameters used by every collision benchmark.
pub const PARAMS: Params = Params {
    width: 200,
    height: 100,
    samples: 10,
    max_depth: 10,
    random_seed: false,
    use_bvh: false,
    soa: SoaMode::Auto,
};
