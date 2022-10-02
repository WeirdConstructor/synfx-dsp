// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! A collection of wave shaping functions.

use std::simd::f32x4;
use std::simd::StdFloat;

// Ported from LMMS under GPLv2
// * DspEffectLibrary.h - library with template-based inline-effects
// * Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
//
// Original source seems to be musicdsp.org, Author: Bram de Jong
// see also: https://www.musicdsp.org/en/latest/Effects/41-waveshaper.html
// Notes:
//     where x (in [-1..1] will be distorted and a is a distortion parameter
//     that goes from 1 to infinity. The equation is valid for positive and
//     negativ values. If a is 1, it results in a slight distortion and with
//     bigger a's the signal get's more funky.
//     A good thing about the shaper is that feeding it with bigger-than-one
//     values, doesn't create strange fx. The maximum this function will reach
//     is 1.2 for a=1.
//
//     f(x,a) = x*(abs(x) + a)/(x^2 + (a-1)*abs(x) + 1)
/// Signal distortion by Bram de Jong.
/// ```text
/// gain:        0.1 - 5.0       default = 1.0
/// threshold:   0.0 - 100.0     default = 0.8
/// i:           signal
/// ```
#[inline]
pub fn f_distort(gain: f32, threshold: f32, i: f32) -> f32 {
    gain * (i * (i.abs() + threshold) / (i * i + (threshold - 1.0) * i.abs() + 1.0))
}

// Ported from LMMS under GPLv2
// * DspEffectLibrary.h - library with template-based inline-effects
// * Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
//
/// Foldback Signal distortion
/// ```text
/// gain:        0.1 - 5.0       default = 1.0
/// threshold:   0.0 - 100.0     default = 0.8
/// i:           signal
/// ```
#[inline]
pub fn f_fold_distort(gain: f32, threshold: f32, i: f32) -> f32 {
    if i >= threshold || i < -threshold {
        gain * ((((i - threshold) % threshold * 4.0).abs() - threshold * 2.0).abs() - threshold)
    } else {
        gain * i
    }
}

/// Cheap 4 channel tanh to make the filter faster.
// Taken from va-filter by Fredemus aka Frederik Halkjær aka RocketPhysician
// https://github.com/Fredemus/va-filter
// Under License GPL-3.0-or-later
//
// from a quick look it looks extremely good, max error of ~0.0002 or .02%
// the error of 1 - tanh_levien^2 as the derivative is about .06%
#[inline]
pub fn tanh_levien(x: f32x4) -> f32x4 {
    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    let a = x + (f32x4::splat(0.16489087) * x3) + (f32x4::splat(0.00985468) * x5);
    // println!("a: {:?}, b: {:?}", a, b);
    a / (f32x4::splat(1.0) + (a * a)).sqrt()
}
