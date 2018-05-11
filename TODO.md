* Calls to modules not inlined (e.g. iterators, rand)
* Rust doesn't support fast floats 
  - see https://internals.rust-lang.org/t/pre-rfc-whats-the-best-way-to-implement-ffast-math/5740/16
  - try out https://gitlab.com/kornelski/ffast-math or do something similar
* SIMD implementation
