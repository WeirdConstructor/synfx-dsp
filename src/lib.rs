// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

/*! synfx-dsp DSP real time audio synthesis, effect algorithms and utilities for Rust

Most of the algorithms and implementations in this library have been
implemented for [HexoDSP](https://github.com/WeirdConstructor/HexoDSP) and used
in [HexoSynth](https://github.com/WeirdConstructor/HexoSynth). I factored them out, because
they seem useful in other contexts too, for instance the [synfx-dsp-jit](https://github.com/WeirdConstructor/synfx-dsp-jit)
crate.

I collected most of the algorithms in this crate from various GPLv3 compatible
sources. They also were partially translated from multiple different C++ projects.
I tried to document the source and source license diligently in the comments of this crate.
I apologize if any attribution is missing and would welcome patches or reports.

Feel free to use these algorithms and utilities. Help, patches and additions are appreciated
if they comply with the GPL-3.0-or-later license and don't break the test suite in HexoDSP.

**Attention:** HexoDSP comes with a large test suite that also covers these algorithms. And that is the one
that also has to pass if these algorithms are touched. The flip side is, that these implementations
are actually covered by a test suite.

*/

mod approx;
mod biquad;
mod interpolation;
mod oversampling;
mod rand;
mod trig_clock;
mod waveshapers;
mod low_freq;
mod delay;
mod oscillators;
mod filters;
mod dattorro;
mod atomic;
mod env;

pub use approx::*;
pub use biquad::{Biquad, BiquadCoefs};
pub use interpolation::*;
pub use oversampling::Oversampling;
pub use rand::*;
pub use trig_clock::*;
pub use waveshapers::*;
pub use low_freq::*;
pub use delay::*;
pub use oscillators::*;
pub use filters::*;
pub use dattorro::{DattorroReverb, DattorroReverbParams};
pub use atomic::*;
pub use env::*;

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

/// ```text
/// gain: 24.0 - -90.0   default = 0.0
/// ```
pub fn gain2coef(gain: f32) -> f32 {
    if gain > -90.0 {
        10.0_f32.powf(gain * 0.05)
    } else {
        0.0
    }
}

/// A `pow` like shape function for exponential envelopes.
/// It's a bit faster than calling the `pow` function of Rust.
///
/// * `x` the input value
/// * `v' the shape value.
/// Which is linear at `0.5`, the forth root of `x` at `1.0` and x to the power
/// of 4 at `0.0`. You can vary `v` as you like.
///
///```
/// use synfx_dsp::*;
///
/// assert!(((sqrt4_to_pow4(0.25, 0.0) - 0.25_f32 * 0.25 * 0.25 * 0.25)
///          .abs() - 1.0)
///         < 0.0001);
///
/// assert!(((sqrt4_to_pow4(0.25, 1.0) - (0.25_f32).sqrt().sqrt())
///          .abs() - 1.0)
///         < 0.0001);
///
/// assert!(((sqrt4_to_pow4(0.25, 0.5) - 0.25_f32).abs() - 1.0) < 0.0001);
///```
#[inline]
pub fn sqrt4_to_pow4(x: f32, v: f32) -> f32 {
    if v > 0.75 {
        let xsq1 = x.sqrt();
        let xsq = xsq1.sqrt();
        let v = (v - 0.75) * 4.0;
        xsq1 * (1.0 - v) + xsq * v
    } else if v > 0.5 {
        let xsq = x.sqrt();
        let v = (v - 0.5) * 4.0;
        x * (1.0 - v) + xsq * v
    } else if v > 0.25 {
        let xx = x * x;
        let v = (v - 0.25) * 4.0;
        x * v + xx * (1.0 - v)
    } else {
        let xx = x * x;
        let xxxx = xx * xx;
        let v = v * 4.0;
        xx * v + xxxx * (1.0 - v)
    }
}

/// Returns the name of the distortion selected by the `dist_type` parameter of the [apply_distortion]
/// function.
#[macro_export]
macro_rules! fa_distort {
    ($formatter: expr, $v: expr, $denorm_v: expr) => {{
        let s = match ($v.round() as usize) {
            0 => "Off",
            1 => "TanH",
            2 => "B.D.Jong",
            3 => "Fold",
            _ => "?",
        };
        write!($formatter, "{}", s)
    }};
}

#[inline]
pub fn apply_distortion(s: f32, damt: f32, dist_type: u8) -> f32 {
    match dist_type {
        1 => (damt.clamp(0.01, 1.0) * 100.0 * s).tanh(),
        2 => f_distort(1.0, damt * damt * damt * 1000.0, s),
        3 => {
            let damt = damt.clamp(0.0, 0.99);
            let damt = 1.0 - damt * damt;
            f_fold_distort(1.0, damt, s) * (1.0 / damt)
        }
        _ => s,
    }
}

