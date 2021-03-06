# Rust Ray Tracing in a Weekend

This is a Rust implementation of the Raytracer described in Peter Shirley's [Ray Tracing in One Weekend](https://in1weekend.blogspot.com/2016/01/ray-tracing-in-one-weekend.html). It is indended for my own learning purposes. The initial implementation is faithfully following the book implementation. My intention is to use this as a test bed for playing with threading, SIMD and general optimizations in Rust and also to complete follow up books to Ray Tracing in a Weekend.

I found the book via Aras Pranckevičius' [blog series](http://aras-p.info/blog/2018/03/28/Daily-Pathtracer-Part-0-Intro/) on his [toy path tracer](https://github.com/aras-p/ToyPathTracer) experiments which was my inspiration for doing this, thanks Aras!

## Compiling and running

Requires Rust 1.27 for SIMD support.

The easiest way to build and run the path tracer use the command:

```
cargo run --release
```

Using nightly will give slightly better performance due to use of `std::intrinsics::cttz`. To build with core intrinsics enabled make sure you have a nightly build of rustc installed and use the command:

```
cargo +nightly run --release --features core_intrinsics
```

If you build without the `--release` flag the application will be very slow!

## License
[license]: #license

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
