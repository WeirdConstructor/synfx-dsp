// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.


//! Interpolated delay line implementation and all-pass/comb filter implementations based on that.

use crate::{Flt, f};
use crate::cubic_interpolate;

/// Default size of the delay buffer: 5 seconds at 8 times 48kHz
const DEFAULT_DELAY_BUFFER_SAMPLES: usize = 8 * 48000 * 5;

/// This is a delay buffer/line with linear and cubic interpolation.
///
/// It's the basic building block underneath the all-pass filter, comb filters and delay effects.
/// You can use linear and cubic and no interpolation to access samples in the past. Either
/// by sample offset or time (millisecond) based.
#[derive(Debug, Clone, Default)]
pub struct DelayBuffer<F: Flt> {
    data: Vec<F>,
    wr: usize,
    srate: F,
}

impl<F: Flt> DelayBuffer<F> {
    /// Creates a delay buffer with about 5 seconds of capacity at 8*48000Hz sample rate.
    pub fn new() -> Self {
        Self { data: vec![f(0.0); DEFAULT_DELAY_BUFFER_SAMPLES], wr: 0, srate: f(44100.0) }
    }

    /// Creates a delay buffer with the given amount of samples capacity.
    pub fn new_with_size(size: usize) -> Self {
        Self { data: vec![f(0.0); size], wr: 0, srate: f(44100.0) }
    }

    /// Sets the sample rate that is used for milliseconds => sample conversion.
    pub fn set_sample_rate(&mut self, srate: F) {
        self.srate = srate;
    }

    /// Reset the delay buffer contents and write position.
    pub fn reset(&mut self) {
        self.data.fill(f(0.0));
        self.wr = 0;
    }

    /// Feed one sample into the delay line and increment the write pointer.
    /// Please note: For sample accurate feedback you need to retrieve the
    /// output of the delay line before feeding in a new signal.
    #[inline]
    pub fn feed(&mut self, input: F) {
        self.data[self.wr] = input;
        self.wr = (self.wr + 1) % self.data.len();
    }

    /// Combines [DelayBuffer::cubic_interpolate_at] and [DelayBuffer::feed]
    /// into one convenient function.
    #[inline]
    pub fn next_cubic(&mut self, delay_time_ms: F, input: F) -> F {
        let res = self.cubic_interpolate_at(delay_time_ms);
        self.feed(input);
        res
    }

    /// Combines [DelayBuffer::linear_interpolate_at] and [DelayBuffer::feed]
    /// into one convenient function.
    #[inline]
    pub fn next_linear(&mut self, delay_time_ms: F, input: F) -> F {
        let res = self.linear_interpolate_at(delay_time_ms);
        self.feed(input);
        res
    }

    /// Combines [DelayBuffer::nearest_at] and [DelayBuffer::feed]
    /// into one convenient function.
    #[inline]
    pub fn next_nearest(&mut self, delay_time_ms: F, input: F) -> F {
        let res = self.nearest_at(delay_time_ms);
        self.feed(input);
        res
    }

    /// Shorthand for [DelayBuffer::cubic_interpolate_at].
    #[inline]
    pub fn tap_c(&self, delay_time_ms: F) -> F {
        self.cubic_interpolate_at(delay_time_ms)
    }

    /// Shorthand for [DelayBuffer::cubic_interpolate_at].
    #[inline]
    pub fn tap_n(&self, delay_time_ms: F) -> F {
        self.nearest_at(delay_time_ms)
    }

    /// Shorthand for [DelayBuffer::cubic_interpolate_at].
    #[inline]
    pub fn tap_l(&self, delay_time_ms: F) -> F {
        self.linear_interpolate_at(delay_time_ms)
    }

    /// Fetch a sample from the delay buffer at the given tim with linear interpolation.
    ///
    /// * `delay_time_ms` - Delay time in milliseconds.
    #[inline]
    pub fn linear_interpolate_at(&self, delay_time_ms: F) -> F {
        self.linear_interpolate_at_s((delay_time_ms * self.srate) / f(1000.0))
    }

    /// Fetch a sample from the delay buffer at the given offset with linear interpolation.
    ///
    /// * `s_offs` - Sample offset in samples.
    #[inline]
    pub fn linear_interpolate_at_s(&self, s_offs: F) -> F {
        let data = &self.data[..];
        let len = data.len();
        let offs = s_offs.floor().to_usize().unwrap_or(0) % len;
        let fract = s_offs.fract();

        // one extra offset, because feed() advances self.wr to the next writing position!
        let i = (self.wr + len) - (offs + 1);
        let x0 = data[i % len];
        let x1 = data[(i - 1) % len];

        let res = x0 + fract * (x1 - x0);
        //d// eprintln!(
        //d//     "INTERP: {:6.4} x0={:6.4} x1={:6.4} fract={:6.4} => {:6.4}",
        //d//     s_offs.to_f64().unwrap_or(0.0),
        //d//     x0.to_f64().unwrap(),
        //d//     x1.to_f64().unwrap(),
        //d//     fract.to_f64().unwrap(),
        //d//     res.to_f64().unwrap(),
        //d// );
        res
    }

    /// Fetch a sample from the delay buffer at the given time with cubic interpolation.
    ///
    /// * `delay_time_ms` - Delay time in milliseconds.
    #[inline]
    pub fn cubic_interpolate_at(&self, delay_time_ms: F) -> F {
        self.cubic_interpolate_at_s((delay_time_ms * self.srate) / f(1000.0))
    }

    /// Fetch a sample from the delay buffer at the given offset with cubic interpolation.
    ///
    /// * `s_offs` - Sample offset in samples into the past of the [DelayBuffer]
    /// from the current write (or the "now") position.
    #[inline]
    pub fn cubic_interpolate_at_s(&self, s_offs: F) -> F {
        let data = &self.data[..];
        let len = data.len();
        let offs = s_offs.floor().to_usize().unwrap_or(0) % len;
        let fract = s_offs.fract();

        // (offs + 1) offset for compensating that self.wr points to the next
        // unwritten position.
        // Additional (offs + 1 + 1) offset for cubic_interpolate, which
        // interpolates into the past through the delay buffer.
        let i = (self.wr + len) - (offs + 2);
        let res = cubic_interpolate(data, len, i, f::<F>(1.0) - fract);
        //        eprintln!(
        //            "cubic at={} ({:6.4}) res={:6.4}",
        //            i % len,
        //            s_offs.to_f64().unwrap(),
        //            res.to_f64().unwrap()
        //        );
        res
    }

    /// Fetch a sample from the delay buffer at the given time without any interpolation.
    ///
    /// * `delay_time_ms` - Delay time in milliseconds.
    #[inline]
    pub fn nearest_at(&self, delay_time_ms: F) -> F {
        let len = self.data.len();
        let offs = ((delay_time_ms * self.srate) / f(1000.0)).floor().to_usize().unwrap_or(0) % len;
        // (offs + 1) one extra offset, because feed() advances
        // self.wr to the next writing position!
        let idx = ((self.wr + len) - (offs + 1)) % len;
        self.data[idx]
    }

    /// Fetch a sample from the delay buffer at the given number of samples in the past.
    #[inline]
    pub fn at(&self, delay_sample_count: usize) -> F {
        let len = self.data.len();
        // (delay_sample_count + 1) one extra offset, because feed() advances self.wr to
        // the next writing position!
        let idx = ((self.wr + len) - (delay_sample_count + 1)) % len;
        self.data[idx]
    }
}

/// Default size of the delay buffer: 1 seconds at 8 times 48kHz
const DEFAULT_ALLPASS_COMB_SAMPLES: usize = 8 * 48000;

/// An all-pass filter based on a delay line.
#[derive(Debug, Clone, Default)]
pub struct AllPass<F: Flt> {
    delay: DelayBuffer<F>,
}

impl<F: Flt> AllPass<F> {
    /// Creates a new all-pass filter with about 1 seconds space for samples.
    pub fn new() -> Self {
        Self { delay: DelayBuffer::new_with_size(DEFAULT_ALLPASS_COMB_SAMPLES) }
    }

    /// Set the sample rate for millisecond based access.
    pub fn set_sample_rate(&mut self, srate: F) {
        self.delay.set_sample_rate(srate);
    }

    /// Reset the internal delay buffer.
    pub fn reset(&mut self) {
        self.delay.reset();
    }

    /// Access the internal delay at the given amount of milliseconds in the past.
    #[inline]
    pub fn delay_tap_n(&self, time_ms: F) -> F {
        self.delay.tap_n(time_ms)
    }

    /// Retrieve the next (cubic interpolated) sample from the all-pass
    /// filter while feeding in the next.
    ///
    /// * `time_ms` - Delay time in milliseconds.
    /// * `g` - Feedback factor (usually something around 0.7 is common)
    /// * `v` - The new input sample to feed the filter.
    #[inline]
    pub fn next(&mut self, time_ms: F, g: F, v: F) -> F {
        let s = self.delay.cubic_interpolate_at(time_ms);
        let input = v + -g * s;
        self.delay.feed(input);
        input * g + s
    }

    /// Retrieve the next (linear interpolated) sample from the all-pass
    /// filter while feeding in the next.
    ///
    /// * `time_ms` - Delay time in milliseconds.
    /// * `g` - Feedback factor (usually something around 0.7 is common)
    /// * `v` - The new input sample to feed the filter.
    #[inline]
    pub fn next_linear(&mut self, time_ms: F, g: F, v: F) -> F {
        let s = self.delay.linear_interpolate_at(time_ms);
        let input = v + -g * s;
        self.delay.feed(input);
        input * g + s
    }
}

#[derive(Debug, Clone)]
pub struct Comb {
    delay: DelayBuffer<f32>,
}

impl Comb {
    pub fn new() -> Self {
        Self { delay: DelayBuffer::new_with_size(DEFAULT_ALLPASS_COMB_SAMPLES) }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.delay.set_sample_rate(srate);
    }

    pub fn reset(&mut self) {
        self.delay.reset();
    }

    #[inline]
    pub fn delay_tap_c(&self, time_ms: f32) -> f32 {
        self.delay.tap_c(time_ms)
    }

    #[inline]
    pub fn delay_tap_n(&self, time_ms: f32) -> f32 {
        self.delay.tap_n(time_ms)
    }

    #[inline]
    pub fn next_feedback(&mut self, time: f32, g: f32, v: f32) -> f32 {
        let s = self.delay.cubic_interpolate_at(time);
        let v = v + s * g;
        self.delay.feed(v);
        v
    }

    #[inline]
    pub fn next_feedforward(&mut self, time: f32, g: f32, v: f32) -> f32 {
        let s = self.delay.next_cubic(time, v);
        v + s * g
    }
}

