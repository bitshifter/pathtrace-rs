use std::fs::File;
use std::io::Write;

fn main() {
    let nx = 200;
    let ny = 100;
    let mut out = File::create("output.ppm").expect("Failed to create output.ppm");
    write!(out, "P3\n{} {} \n255\n", nx, ny).expect("Failed to write to output.ppm");
    for j in 0..ny {
        for i in 0..nx {
            let r = i as f32 / nx as f32;
            let g = (ny - 1 - j) as f32 / ny as f32;
            let b = 0.2;
            let ir = (255.99 * r) as i32;
            let ig = (255.99 * g) as i32;
            let ib = (255.99 * b) as i32;
            write!(out, "{} {} {}\n", ir, ig, ib).expect("Failed to write to output.ppm");
        }
    }
}
