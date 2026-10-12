//! How to call a kernel without paying for the level lookup on every call.
//!
//! `dispatch!` picks the kernel again on every call, which is what you want when a level is used
//! once. When the same kernel runs over many batches, select it once with `dispatch_entry!` and
//! call the entry instead.
//!
//! `cargo run --release --example dispatch_entry`

use glam_fearless::{Level, Vec3xN, dispatch, dispatch_entry, prelude::*};

/// The kernel, as a plain generic function: no `#[simd]` attribute, because the selection is what
/// `dispatch_entry!` adds.
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

fn main() {
    const LANES: usize = 16;

    // One detection and one selection, reused for every call.
    let length = LengthEntry::new();
    println!("selected for {:?}", length.level());

    for batch in 0..3 {
        let x: Vec<f32> = (0..LANES)
            .map(|i| (batch * LANES + i) as f32 + 1.0)
            .collect();
        let y = vec![4.0f32; LANES];
        let z = vec![12.0f32; LANES];
        let mut out = vec![0.0f32; LANES];

        // Safe: the entry carries the level it was selected for.
        length.call([&x, &y, &z], &mut out);
        println!("batch {batch}: {:.1} {:.1} ...", out[0], out[1]);
    }

    // `lazy()` does the same without keeping the entry around. It resolves on the first call and
    // then pays a predictable atomic load per call, which is a few instructions.
    let x = vec![3.0f32; LANES];
    let y = vec![4.0f32; LANES];
    let z = vec![12.0f32; LANES];
    let mut out = vec![0.0f32; LANES];
    LengthEntry::lazy().call([&x, &y, &z], &mut out);
    println!("lazy: {:.1}", out[0]);

    // A caller that processes many batches can do better still: one `dispatch!` around the whole
    // loop, with the plain generic body inside it. That pays for the selection once per batch of
    // batches instead of once per kernel call.
    let batches: Vec<[Vec<f32>; 3]> = (0..3)
        .map(|batch| {
            [
                (0..LANES).map(|i| (batch * LANES + i) as f32).collect(),
                y.clone(),
                z.clone(),
            ]
        })
        .collect();
    let level = Level::new();
    dispatch!(level, simd => {
        for batch in &batches {
            let refs = [batch[0].as_slice(), batch[1].as_slice(), batch[2].as_slice()];
            length_body(simd, refs, &mut out);
        }
    });
    println!("one dispatch for {} batches: {:.1}", batches.len(), out[0]);
}
