// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.
//
// The implementation of this Biquad Filter has been adapted from
// SamiPerttu, Copyright (c) 2020, under the MIT License.
// See also: https://github.com/SamiPerttu/fundsp/blob/master/src/filter.rs
//
// You will find a float type agnostic version in SamiPerttu's code.
// I converted this to pure f32 for no good reason, other than making
// the code more readable (for me).

//! A biquad filter implementation.
///
/// It is unfortunately still missing some coefficient calculations for some types of filters.

use std::f32::consts::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct BiquadCoefs {
    pub a1: f32,
    pub a2: f32,
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
}

// TODO:
// https://github.com/VCVRack/Befaco/blob/v1/src/ChowDSP.hpp#L339
// more coeffs from there ^^^^^^^^^^^^^ ?
impl BiquadCoefs {
    #[inline]
    pub fn new(b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) -> Self {
        Self { b0, b1, b2, a1, a2 }
    }

    /// Returns settings for a Butterworth lowpass filter.
    /// Cutoff is the -3 dB point of the filter in Hz.
    #[inline]
    pub fn butter_lowpass(sample_rate: f32, cutoff: f32) -> BiquadCoefs {
        let f = (cutoff * PI / sample_rate).tan();
        let a0r = 1.0 / (1.0 + SQRT_2 * f + f * f);
        let a1 = (2.0 * f * f - 2.0) * a0r;
        let a2 = (1.0 - SQRT_2 * f + f * f) * a0r;
        let b0 = f * f * a0r;
        let b1 = 2.0 * b0;
        let b2 = b0;
        BiquadCoefs { a1, a2, b0, b1, b2 }
    }

    /// Returns the Q for cascading a butterworth filter:
    pub fn calc_cascaded_butter_q(order: usize, casc_idx: usize) -> f32 {
        let order = order as f32;
        let casc_idx = casc_idx as f32;

        let b = -2.0 * ((2.0 * casc_idx + order - 1.0) * PI / (2.0 * order)).cos();

        1.0 / b
    }

    /// Returns settings for a lowpass filter with a specific q
    #[inline]
    pub fn lowpass(sample_rate: f32, q: f32, cutoff: f32) -> BiquadCoefs {
        let f = (cutoff * PI / sample_rate).tan();
        let a0r = 1.0 / (1.0 + f / q + f * f);

        /*
        float norm = 1.f / (1.f + K / Q + K * K);
        this->b[0] = K * K * norm;
        this->b[1] = 2.f * this->b[0];
        this->b[2] = this->b[0];
        this->a[1] = 2.f * (K * K - 1.f) * norm;
        this->a[2] = (1.f - K / Q + K * K) * norm;
        */

        let b0 = f * f * a0r;
        let b1 = 2.0 * b0;
        let b2 = b0;
        let a1 = 2.0 * (f * f - 1.0) * a0r;
        let a2 = (1.0 - f / q + f * f) * a0r;

        BiquadCoefs { a1, a2, b0, b1, b2 }
    }

    /// Returns settings for a constant-gain bandpass resonator.
    /// The center frequency is given in Hz.
    /// Bandwidth is the difference in Hz between -3 dB points of the filter response.
    /// The overall gain of the filter is independent of bandwidth.
    pub fn resonator(sample_rate: f32, center: f32, bandwidth: f32) -> BiquadCoefs {
        let r = (-PI * bandwidth / sample_rate).exp();
        let a1 = -2.0 * r * (TAU * center / sample_rate).cos();
        let a2 = r * r;
        let b0 = (1.0 - r * r).sqrt() * 0.5;
        let b1 = 0.0;
        let b2 = -b0;
        BiquadCoefs { a1, a2, b0, b1, b2 }
    }

    //    /// Frequency response at frequency `omega` expressed as fraction of sampling rate.
    //    pub fn response(&self, omega: f64) -> Complex64 {
    //        let z1 = Complex64::from_polar(1.0, -TAU * omega);
    //        let z2 = Complex64::from_polar(1.0, -2.0 * TAU * omega);
    //        (re(self.b0) + re(self.b1) * z1 + re(self.b2) * z2)
    //            / (re(1.0) + re(self.a1) * z1 + re(self.a2) * z2)
    //    }
}

/// 2nd order IIR filter implemented in normalized Direct Form I.
#[derive(Debug, Copy, Clone, Default)]
pub struct Biquad {
    coefs: BiquadCoefs,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn new_with(b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) -> Self {
        let mut s = Self::new();
        s.set_coefs(BiquadCoefs::new(b0, b1, b2, a1, a2));
        s
    }

    #[inline]
    pub fn coefs(&self) -> &BiquadCoefs {
        &self.coefs
    }

    #[inline]
    pub fn set_coefs(&mut self, coefs: BiquadCoefs) {
        self.coefs = coefs;
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    #[inline]
    pub fn tick(&mut self, input: f32) -> f32 {
        let x0 = input;
        let y0 = self.coefs.b0 * x0 + self.coefs.b1 * self.x1 + self.coefs.b2 * self.x2
            - self.coefs.a1 * self.y1
            - self.coefs.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        y0

        // Transposed Direct Form II would be:
        //   y0 = b0 * x0 + s1
        //   s1 = s2 + b1 * x0 - a1 * y0
        //   s2 = b2 * x0 - a2 * y0
    }
}

#[derive(Copy, Clone)]
pub struct ButterLowpass {
    biquad: Biquad,
    sample_rate: f32,
    cutoff: f32,
}

#[allow(dead_code)]
impl ButterLowpass {
    pub fn new(sample_rate: f32, cutoff: f32) -> Self {
        let mut this = ButterLowpass { biquad: Biquad::new(), sample_rate, cutoff: 0.0 };
        this.set_cutoff(cutoff);
        this
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.biquad.set_coefs(BiquadCoefs::butter_lowpass(self.sample_rate, cutoff));
        self.cutoff = cutoff;
    }

    fn set_sample_rate(&mut self, srate: f32) {
        self.sample_rate = srate;
        self.reset();
        self.biquad.reset();
        self.set_cutoff(self.cutoff);
    }

    fn reset(&mut self) {
        self.biquad.reset();
        self.set_cutoff(self.cutoff);
    }

    #[inline]
    fn tick(&mut self, input: f32) -> f32 {
        self.biquad.tick(input)
    }
}
