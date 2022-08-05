// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! Various interpolation related functions.

use crate::{Flt, f};
use crate::tanh_approx_drive;

/// Linear crossfade.
///
/// * `v1` - signal 1, range -1.0 to 1.0
/// * `v2` - signal 2, range -1.0 to 1.0
/// * `mix` - mix position, range 0.0 to 1.0, mid is at 0.5
#[inline]
pub fn crossfade<F: Flt>(v1: F, v2: F, mix: F) -> F {
    v1 * (f::<F>(1.0) - mix) + v2 * mix
}

/// Linear crossfade with clipping the `v2` result.
///
/// This crossfade actually does clip the `v2` signal to the -1.0 to 1.0
/// range. This is useful for Dry/Wet of plugins that might go beyond the
/// normal signal range.
///
/// * `v1` - signal 1, range -1.0 to 1.0
/// * `v2` - signal 2, range -1.0 to 1.0
/// * `mix` - mix position, range 0.0 to 1.0, mid is at 0.5
#[inline]
pub fn crossfade_clip<F: Flt>(v1: F, v2: F, mix: F) -> F {
    v1 * (f::<F>(1.0) - mix) + (v2 * mix).min(f::<F>(1.0)).max(f::<F>(-1.0))
}

/// Linear (f32) crossfade with driving the `v2` result through a tanh().
///
/// * `v1` - signal 1, range -1.0 to 1.0
/// * `v2` - signal 2, range -1.0 to 1.0
/// * `mix` - mix position, range 0.0 to 1.0, mid is at 0.5
#[inline]
pub fn crossfade_drive_tanh(v1: f32, v2: f32, mix: f32) -> f32 {
    v1 * (1.0 - mix) + tanh_approx_drive(v2 * mix * 0.111, 0.95) * 0.9999
}

/// Constant power crossfade.
///
/// * `v1` - signal 1, range -1.0 to 1.0
/// * `v2` - signal 2, range -1.0 to 1.0
/// * `mix` - mix position, range 0.0 to 1.0, mid is at 0.5
#[inline]
pub fn crossfade_cpow(v1: f32, v2: f32, mix: f32) -> f32 {
    let s1 = (mix * std::f32::consts::FRAC_PI_2).sin();
    let s2 = ((1.0 - mix) * std::f32::consts::FRAC_PI_2).sin();
    v1 * s2 + v2 * s1
}

const CROSS_LOG_MIN: f32 = -13.815510557964274; // (0.000001_f32).ln();
const CROSS_LOG_MAX: f32 = 0.0; // (1.0_f32).ln();

/// Logarithmic crossfade.
///
/// * `v1` - signal 1, range -1.0 to 1.0
/// * `v2` - signal 2, range -1.0 to 1.0
/// * `mix` - mix position, range 0.0 to 1.0, mid is at 0.5
#[inline]
pub fn crossfade_log(v1: f32, v2: f32, mix: f32) -> f32 {
    let x = (mix * (CROSS_LOG_MAX - CROSS_LOG_MIN) + CROSS_LOG_MIN).exp();
    crossfade(v1, v2, x)
}

/// Exponential crossfade.
///
/// * `v1` - signal 1, range -1.0 to 1.0
/// * `v2` - signal 2, range -1.0 to 1.0
/// * `mix` - mix position, range 0.0 to 1.0, mid is at 0.5
#[inline]
pub fn crossfade_exp(v1: f32, v2: f32, mix: f32) -> f32 {
    crossfade(v1, v2, mix * mix)
}

/// Apply linear interpolation between the value a and b.
///
/// * `a` - value at x=0.0
/// * `b` - value at x=1.0
/// * `x` - value between 0.0 and 1.0 to blend between `a` and `b`.
#[inline]
pub fn lerp(x: f32, a: f32, b: f32) -> f32 {
    (a * (1.0 - x)) + (b * x)
}

/// Apply 64bit linear interpolation between the value a and b.
///
/// * `a` - value at x=0.0
/// * `b` - value at x=1.0
/// * `x` - value between 0.0 and 1.0 to blend between `a` and `b`.
#[inline]
pub fn lerp64(x: f64, a: f64, b: f64) -> f64 {
    (a * (1.0 - x)) + (b * x)
}

/// Hermite / Cubic interpolation of a buffer full of samples at the given _index_.
/// _len_ is the buffer length to consider and wrap the index into. And _fract_ is the
/// fractional part of the index.
///
/// This function is generic over f32 and f64. That means you can use your preferred float size.
///
/// Commonly used like this:
///
///```
/// use hexodsp::dsp::helpers::cubic_interpolate;
///
/// let buf : [f32; 9] = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2];
/// let pos = 3.3_f32;
///
/// let i = pos.floor() as usize;
/// let f = pos.fract();
///
/// let res = cubic_interpolate(&buf[..], buf.len(), i, f);
/// assert!((res - 0.67).abs() < 0.2_f32);
///```
#[inline]
pub fn cubic_interpolate<F: Flt>(data: &[F], len: usize, index: usize, fract: F) -> F {
    let index = index + len;
    // Hermite interpolation, take from
    // https://github.com/eric-wood/delay/blob/main/src/delay.rs#L52
    //
    // Thanks go to Eric Wood!
    //
    // For the interpolation code:
    // MIT License, Copyright (c) 2021 Eric Wood
    let xm1 = data[(index - 1) % len];
    let x0 = data[index % len];
    let x1 = data[(index + 1) % len];
    let x2 = data[(index + 2) % len];

    let c = (x1 - xm1) * f(0.5);
    let v = x0 - x1;
    let w = c + v;
    let a = w + v + (x2 - x0) * f(0.5);
    let b_neg = w + a;

    let res = (((a * fract) - b_neg) * fract + c) * fract + x0;

    // let rr2 =
    //     x0 + f::<F>(0.5) * fract * (
    //         x1 - xm1 + fract * (
    //             f::<F>(4.0) * x1
    //             + f::<F>(2.0) * xm1
    //             - f::<F>(5.0) * x0
    //             - x2
    //             + fract * (f::<F>(3.0) * (x0 - x1) - xm1 + x2)));

    // eprintln!(
    //     "index={} fract={:6.4} xm1={:6.4} x0={:6.4} x1={:6.4} x2={:6.4} = {:6.4} <> {:6.4}",
    //     index, fract.to_f64().unwrap(), xm1.to_f64().unwrap(), x0.to_f64().unwrap(), x1.to_f64().unwrap(), x2.to_f64().unwrap(),
    //     res.to_f64().unwrap(),
    //     rr2.to_f64().unwrap()
    // );

    // eprintln!(
    //     "index={} fract={:6.4} xm1={:6.4} x0={:6.4} x1={:6.4} x2={:6.4} = {:6.4}",
    //     index, fract.to_f64().unwrap(), xm1.to_f64().unwrap(), x0.to_f64().unwrap(), x1.to_f64().unwrap(), x2.to_f64().unwrap(),
    //     res.to_f64().unwrap(),
    // );

    res
}

