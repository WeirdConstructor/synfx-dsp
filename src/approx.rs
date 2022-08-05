// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

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
