//! Calling a kernel without looking the level up again on every call.
//!
//! [`dispatch!`](crate::dispatch) picks the kernel again on every call. That suits a level used
//! once, but a caller that runs the same kernel over many batches pays for the selection each
//! time: a match, a closure and `Simd::vectorize` all sit in front of the kernel.
//! [`dispatch_entry!`](crate::dispatch_entry) selects once and keeps the result, so what is left
//! on the per-call path is one indirect call — or no call at all, when the build already enables
//! the level's features.
//!
//! Measured on Zen 2 (AVX2), 16-element batches, kernel = per-lane length of three SoA component
//! slices. Times are ns per call; the instruction counts are from `perf stat` in the first column's
//! build and are steady, while the times move a few percent between runs on this machine.
//!
//! | entry | insn/call | caller built without AVX2 | caller built with AVX2 |
//! |---|---|---|---|
//! | `dispatch!` per call | 104 | 6.18 | 5.44 |
//! | entry held by the caller | 80 | 5.67 | 4.10 |
//! | entry resolved on first use | 83 | 5.88 | 4.25 |
//! | hand-written `#[target_feature]` loop | 85 | 4.23 | 4.30 |
//!
//! The entry is the cheapest way to call the kernel once per batch in both builds: 8% better than
//! `dispatch!` per call when the caller is built without the level, and 25% better when it is built
//! with it, where it also beats the hand-written loop because that one pays for an out-of-line
//! call. The saving is fixed per call, so it matters for small batches and fades away once the
//! kernel is throughput bound; wrapping a whole batch in one `dispatch!` is cheaper still.
//!
//! Resolving the entry on first use costs one predictable atomic load and a branch: 3 instructions,
//! and 0.1-0.2 ns above.

/// Define an entry point for one kernel body.
///
/// Generates a struct that keeps the level it was built for alongside a callable kernel, plus a
/// safe `call` method. Build it once and reuse it; that is what keeps the selection out of the
/// per-call path.
///
/// The kernel must be a plain generic function over `S: Simd`, not a `#[simd]` kernel: a `#[simd]`
/// kernel looks the level up again on every call, which is the cost this exists to avoid.
///
/// If the crate that expands the macro is compiled with a level's target features, `call` uses
/// that level directly and the kernel inlines into the caller, exactly as `dispatch!` would. The
/// level argument still has to agree, so the values never change; only the call path does.
///
/// ```
/// use glam_fearless::{prelude::*, Level, Vec3xN};
///
/// #[inline(always)]
/// fn length_body<S: Simd>(simd: S, a: [&[f32]; 3], out: &mut [f32]) {
///     let n = Vec3xN::<S>::LANES;
///     let ins = a[0]
///         .chunks_exact(n)
///         .zip(a[1].chunks_exact(n))
///         .zip(a[2].chunks_exact(n));
///     for (o, ((x, y), z)) in out.chunks_mut(n).zip(ins) {
///         Vec3xN::from_slice(simd, x, y, z).length().store_slice(o);
///     }
/// }
///
/// glam_fearless::dispatch_entry! {
///     /// Per-lane length of three SoA component slices.
///     pub struct LengthEntry(level: Level) -> fn(a: [&[f32]; 3], out: &mut [f32]) = length_body;
/// }
///
/// // Selected once, then called as often as needed.
/// let length = LengthEntry::new();
/// let (x, y, z) = (vec![3.0f32; 16], vec![4.0f32; 16], vec![12.0f32; 16]);
/// let mut out = vec![0.0f32; 16];
/// length.call([&x, &y, &z], &mut out);
/// assert!(out.iter().all(|v| *v == 13.0));
///
/// // Or leave the selection to the type, for callers that do not want to hold one.
/// LengthEntry::lazy().call([&x, &y, &z], &mut out);
/// assert!(out.iter().all(|v| *v == 13.0));
/// ```
#[macro_export]
macro_rules! dispatch_entry {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident($level:ident : $level_ty:ty)
            -> fn($($arg:ident : $ty:ty),* $(,)?) = $kernel:path;
    ) => {
        $(#[$attr])*
        $vis struct $name {
            level: $level_ty,
            kernel: unsafe fn($($ty),*),
        }

        impl $name {
            // One function per level, so the entry can be a plain function pointer. The feature
            // lists have to match the ones `fearless_simd` uses for the same level: a callee whose
            // features are not a subset of the caller's cannot be inlined, and a shorter list
            // silently turns every operation into an out-of-line call (measured: 62 instructions
            // per chunk instead of 16). They are copied from `fearless_simd`'s
            // `__fearless_simd_kernel_target_fn!`, and the dispatch benches fail visibly if the two
            // drift apart.
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            #[inline] // can't be `inline(always)` with target features
            #[target_feature(
                enable = "fxsr,adx,aes,avx512bitalg,avx512bw,avx512cd,avx512dq,avx512f,avx512ifma,avx512vbmi,avx512vbmi2,avx512vl,avx512vnni,avx512vpopcntdq,bmi1,bmi2,cmpxchg16b,fma,gfni,lzcnt,movbe,pclmulqdq,popcnt,rdrand,rdseed,sha,vaes,vpclmulqdq,xsave,xsavec,xsaveopt,xsaves"
            )]
            unsafe fn avx512($($arg: $ty),*) {
                $kernel(
                    unsafe { <$crate::fearless_simd::Avx512>::assume_supported() }
                    $(, $arg)*
                )
            }

            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            #[inline] // as above
            #[target_feature(
                enable = "fxsr,avx2,bmi1,bmi2,cmpxchg16b,f16c,fma,lzcnt,movbe,popcnt,xsave"
            )]
            unsafe fn avx2($($arg: $ty),*) {
                $kernel(unsafe { <$crate::fearless_simd::Avx2>::assume_supported() } $(, $arg)*)
            }

            // Fallback for levels without an entry of their own, and for every target: the
            // statically supported level. `dispatch!` supplies the target-feature context per
            // call, so this compiles and is correct everywhere, including backends that do not
            // exist yet. The values are the same at any level; only the width changes.
            unsafe fn baseline($($arg: $ty),*) {
                $crate::dispatch!($crate::Level::baseline(), simd => $kernel(simd $(, $arg)*))
            }
            /// Detect the level and select the kernel for it.
            #[must_use]
            pub fn new() -> Self {
                Self::with_level(<$level_ty>::new())
            }

            /// The entry for this process, selected on first use.
            ///
            /// For callers that would rather not keep an entry around. The check costs one
            /// predictable atomic load and a branch per call (measured: 3 instructions, and no
            /// visible time in a 16-element batch), but building the entry with
            /// [`new`](Self::new) and holding it is still the cheaper option for hot loops.
            #[must_use]
            pub fn lazy() -> &'static Self {
                static ENTRY: ::std::sync::LazyLock<$name> =
                    ::std::sync::LazyLock::new(<$name>::new);
                &*ENTRY
            }

            /// Select the kernel for a level the caller already has.
            #[must_use]
            pub fn with_level($level: $level_ty) -> Self {
                #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                {
                    // Widest first; the first match wins.
                    if $level.as_avx512().is_some() {
                        return Self {
                            level: $level,
                            kernel: Self::avx512,
                        };
                    }
                    if $level.as_avx2().is_some() {
                        return Self {
                            level: $level,
                            kernel: Self::avx2,
                        };
                    }
                }

                let _ = &$level;
                Self {
                    level: $level,
                    kernel: Self::baseline,
                }
            }

            /// The level this entry was built for.
            #[must_use]
            pub fn level(&self) -> $level_ty {
                self.level
            }

            /// Run the kernel.
            #[inline(always)]
            pub fn call(&self, $($arg: $ty),*) {
                // If this crate is compiled with a level's features, there is nothing to select:
                // call the kernel for that level directly so it inlines here. Going through the
                // function pointer would stop it from inlining, which is the whole cost in a build
                // like this one. The level still has to match, so the call is only taken when the
                // detected level really is the one the build assumed.
                #[cfg(all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    target_feature = "avx512f"
                ))]
                if self.level.as_avx512().is_some() {
                    // SAFETY: the level reports AVX-512 support and this build enables it.
                    return unsafe { Self::avx512($($arg),*) };
                }
                #[cfg(all(
                    any(target_arch = "x86", target_arch = "x86_64"),
                    target_feature = "avx2"
                ))]
                if self.level.as_avx2().is_some() {
                    // SAFETY: the level reports AVX2 support and this build enables it.
                    return unsafe { Self::avx2($($arg),*) };
                }

                // SAFETY: the pointer was selected from `self.level`, and the level cannot be
                // changed after construction, so it is never called with a different level.
                unsafe { (self.kernel)($($arg),*) }
            }

            /// The selected kernel, for callers that want to avoid the call through the struct.
            /// Carries the same obligation as any entry point: only call it with
            /// [the level it was selected for](Self::level).
            ///
            /// # Safety
            ///
            /// The returned pointer runs with the target features of `self.level()`.
            #[must_use]
            pub fn as_fn(&self) -> unsafe fn($($ty),*) {
                self.kernel
            }
        }

        impl ::core::default::Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}
