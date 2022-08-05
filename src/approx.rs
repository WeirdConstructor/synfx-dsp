// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! Various approximations and faster implementations of trigonometric functions.
///
/// Note: The [fast_cos] and [fast_sin] functions are only barely faster than
/// the Rust builtin `sin` and `cos` functions.

/// Logarithmic table size of the table in [fast_cos] / [fast_sin].
static FAST_COS_TAB_LOG2_SIZE: usize = 9;
/// Table size of the table in [fast_cos] / [fast_sin].
static FAST_COS_TAB_SIZE: usize = 1 << FAST_COS_TAB_LOG2_SIZE; // =512
/// The wave table of [fast_cos] / [fast_sin].
static mut FAST_COS_TAB: [f32; 513] = [0.0; 513];

/// Initializes the cosine wave table for [fast_cos] and [fast_sin].
pub fn init_cos_tab() {
    for i in 0..(FAST_COS_TAB_SIZE + 1) {
        let phase: f32 = (i as f32) * ((std::f32::consts::TAU) / (FAST_COS_TAB_SIZE as f32));
        unsafe {
            // XXX: note: mutable statics can be mutated by multiple
            //      threads: aliasing violations or data races
            //      will cause undefined behavior
            FAST_COS_TAB[i] = phase.cos();
        }
    }
}

/// Internal phase increment/scaling for [fast_cos].
const PHASE_SCALE: f32 = 1.0_f32 / (std::f32::consts::TAU);

/// A faster implementation of cosine. It's not that much faster than
/// Rust's built in cosine function. But YMMV.
///
/// Don't forget to call [init_cos_tab] before using this!
///
///```
/// use hexodsp::dsp::helpers::*;
/// init_cos_tab(); // Once on process initialization.
///
/// // ...
/// assert!((fast_cos(std::f32::consts::PI) - -1.0).abs() < 0.001);
///```
pub fn fast_cos(mut x: f32) -> f32 {
    x = x.abs(); // cosine is symmetrical around 0, let's get rid of negative values

    // normalize range from 0..2PI to 1..2
    let phase = x * PHASE_SCALE;

    let index = FAST_COS_TAB_SIZE as f32 * phase;

    let fract = index.fract();
    let index = index.floor() as usize;

    unsafe {
        // XXX: note: mutable statics can be mutated by multiple
        //      threads: aliasing violations or data races
        //      will cause undefined behavior
        let left = FAST_COS_TAB[index as usize];
        let right = FAST_COS_TAB[index as usize + 1];

        return left + (right - left) * fract;
    }
}

/// A faster implementation of sine. It's not that much faster than
/// Rust's built in sine function. But YMMV.
///
/// Don't forget to call [init_cos_tab] before using this!
///
///```
/// use hexodsp::dsp::helpers::*;
/// init_cos_tab(); // Once on process initialization.
///
/// // ...
/// assert!((fast_sin(0.5 * std::f32::consts::PI) - 1.0).abs() < 0.001);
///```
pub fn fast_sin(x: f32) -> f32 {
    fast_cos(x - (std::f32::consts::PI / 2.0))
}

pub fn square_135(phase: f32) -> f32 {
    fast_sin(phase) + fast_sin(phase * 3.0) / 3.0 + fast_sin(phase * 5.0) / 5.0
}

pub fn square_35(phase: f32) -> f32 {
    fast_sin(phase * 3.0) / 3.0 + fast_sin(phase * 5.0) / 5.0
}


// quickerTanh / quickerTanh64 credits to mopo synthesis library:
// Under GPLv3 or any later.
// Little IO <littleioaudio@gmail.com>
// Matt Tytel <matthewtytel@gmail.com>
pub fn quicker_tanh64(v: f64) -> f64 {
    let square = v * v;
    v / (1.0 + square / (3.0 + square / 5.0))
}

#[inline]
pub fn quicker_tanh(v: f32) -> f32 {
    let square = v * v;
    v / (1.0 + square / (3.0 + square / 5.0))
}

// quickTanh / quickTanh64 credits to mopo synthesis library:
// Under GPLv3 or any later.
// Little IO <littleioaudio@gmail.com>
// Matt Tytel <matthewtytel@gmail.com>
pub fn quick_tanh64(v: f64) -> f64 {
    let abs_v = v.abs();
    let square = v * v;
    let num = v
        * (2.45550750702956
            + 2.45550750702956 * abs_v
            + square * (0.893229853513558 + 0.821226666969744 * abs_v));
    let den =
        2.44506634652299 + (2.44506634652299 + square) * (v + 0.814642734961073 * v * abs_v).abs();

    num / den
}

pub fn quick_tanh(v: f32) -> f32 {
    let abs_v = v.abs();
    let square = v * v;
    let num = v
        * (2.45550750702956
            + 2.45550750702956 * abs_v
            + square * (0.893229853513558 + 0.821226666969744 * abs_v));
    let den =
        2.44506634652299 + (2.44506634652299 + square) * (v + 0.814642734961073 * v * abs_v).abs();

    num / den
}

// Taken from ValleyAudio
// Copyright Dale Johnson
// https://github.dev/ValleyAudio/ValleyRackFree/tree/v2.0
// Under GPLv3 license
pub fn tanh_approx_drive(v: f32, drive: f32) -> f32 {
    let x = v * drive;

    if x < -1.25 {
        -1.0
    } else if x < -0.75 {
        1.0 - (x * (-2.5 - x) - 0.5625) - 1.0
    } else if x > 1.25 {
        1.0
    } else if x > 0.75 {
        x * (2.5 - x) - 0.5625
    } else {
        x
    }
}
