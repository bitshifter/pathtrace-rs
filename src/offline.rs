use crate::{math::linear_to_srgb, perlin::Perlin, presets, scene::Params};
use image;
use rand::{SeedableRng, XorShiftRng};
use std::time::SystemTime;
use typed_arena::Arena;

pub fn render_offline(preset: &str, params: Params) {
    let mut rng = if params.random_seed {
        rand::weak_rng()
    } else {
        const FIXED_SEED: [u32; 4] = [0x193a_6754, 0xa8a7_d469, 0x9783_0e05, 0x113b_a7bb];
        XorShiftRng::from_seed(FIXED_SEED)
    };
    let perlin = Perlin::new(&mut rng);
    let texture_arena = Arena::new();
    let material_arena = Arena::new();
    let (scene, camera) = presets::from_name(
        preset,
        &params,
        &mut rng,
        &texture_arena,
        &material_arena,
        &perlin,
    )
    .expect("unrecognised preset");
    let mut rgb_buffer = vec![(0.0, 0.0, 0.0); (params.width * params.height) as usize];

    let start_time = SystemTime::now();
    let frame_num = 0; // only ever processing 1 frame in offline
    let ray_count = scene.update(&params, &camera, frame_num, &mut rgb_buffer);
    let elapsed = start_time
        .elapsed()
        .expect("SystemTime elapsed time failed");
    let elapsed_secs =
        elapsed.as_secs() as f64 + f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0;

    println!(
        "{:.2}secs {}rays {:.2}Mrays/s",
        elapsed_secs,
        ray_count,
        ray_count as f64 / 1_000_000.0 / elapsed_secs
    );

    let mut image_bytes = Vec::with_capacity(rgb_buffer.len() * 3);
    for row in rgb_buffer.chunks(params.width as usize).rev() {
        for rgb in row {
            let srgb = linear_to_srgb(*rgb);
            image_bytes.push(srgb.0);
            image_bytes.push(srgb.1);
            image_bytes.push(srgb.2);
        }
    }
    image::save_buffer(
        "output.png",
        &image_bytes,
        params.width,
        params.height,
        image::RGB(8),
    )
    .expect("Failed to save output image");
}
