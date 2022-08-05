// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

/*! synfx-dsp Real Time Audio Synthesis and Effect DSP algorithms and utility functions for Rust.

Most of the algorithms and implementations in this library have been
implemented for [HexoDSP](https://github.com/WeirdConstructor/HexoDSP) and used
in [HexoSynth](https://github.com/WeirdConstructor/HexoSynth). I factored them out, because
they seem useful in other contexts too, for instance the [synfx-dsp-jit](https://github.com/WeirdConstructor/synfx-dsp-jit)
crate.

Feel free to use these algorithms and utilities. Help, patches and additions are apprechiated
if they comply with the GPL-3.0-or-later license and don't break the test suite in HexoDSP.

**Attention:** HexoDSP comes with a large test suite that also covers these algorithms. And that is the one
that also has to pass if these algorithms are touched. The flip side is, that these implementations
are actually covered by a test suite.

*/

mod approx;
mod biquad;
mod interpolation;
mod oversampling;

pub use approx::*;
pub use biquad::{Biquad, BiquadCoefs};
pub use interpolation::*;
pub use oversampling::Oversampling;

use num_traits::{cast::FromPrimitive, cast::ToPrimitive, Float, FloatConst};

macro_rules! trait_alias {
    ($name:ident = $base1:ident + $($base2:ident +)+) => {
        pub trait $name: $base1 $(+ $base2)+ { }
        impl<T: $base1 $(+ $base2)+> $name for T { }
    };
}

trait_alias!(Flt = Float + FloatConst + ToPrimitive + FromPrimitive +);

//macro_rules! fc {
//    ($F: ident, $e: expr) => {
//        F::from_f64($e).unwrap()
//    };
//}

#[allow(dead_code)]
#[inline]
fn f<F: Flt>(x: f64) -> F {
    F::from_f64(x).unwrap()
}

#[allow(dead_code)]
#[inline]
fn fclamp<F: Flt>(x: F, mi: F, mx: F) -> F {
    x.max(mi).min(mx)
}

#[allow(dead_code)]
#[inline]
fn fclampc<F: Flt>(x: F, mi: f64, mx: f64) -> F {
    x.max(f(mi)).min(f(mx))
}

/// Converts a midi note (0 to 128) to a frequency
///
///```
/// use synfx_dsp::*;
///
/// assert_eq!(note_to_freq(69.0) as i32, 440);
/// assert_eq!(note_to_freq(69.0 + 12.0) as i32, 880);
/// assert_eq!(note_to_freq(69.0 - 12.0) as i32, 220);
///```
pub fn note_to_freq(note: f32) -> f32 {
    440.0 * (2.0_f32).powf((note - 69.0) / 12.0)
}
