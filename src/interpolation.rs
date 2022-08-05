// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

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

