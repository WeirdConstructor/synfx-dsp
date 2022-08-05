// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! A collection of filters, ranging from simple one poles to more interesting ones.

use crate::{Flt, f};

// one pole lp from valley rack free:
// https://github.com/ValleyAudio/ValleyRackFree/blob/v1.0/src/Common/DSP/OnePoleFilters.cpp
#[inline]
/// Process a very simple one pole 6dB low pass filter.
/// Useful in various applications, from usage in a synthesizer to
/// damping stuff in a reverb/delay.
///
/// * `input`  - Input sample
/// * `freq`   - Frequency between 1.0 and 22000.0Hz
/// * `israte` - 1.0 / samplerate
/// * `z`      - The internal one sample buffer of the filter.
///
///```
///    use synfx_dsp::*;
///
///    let samples  = vec![0.0; 44100];
///    let mut z    = 0.0;
///    let mut freq = 1000.0;
///
///    for s in samples.iter() {
///        let s_out =
///            process_1pole_lowpass(*s, freq, 1.0 / 44100.0, &mut z);
///        // ... do something with the result here.
///    }
///```
pub fn process_1pole_lowpass(input: f32, freq: f32, israte: f32, z: &mut f32) -> f32 {
    let b = (-std::f32::consts::TAU * freq * israte).exp();
    let a = 1.0 - b;
    *z = a * input + *z * b;
    *z
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OnePoleLPF<F: Flt> {
    israte: F,
    a: F,
    b: F,
    freq: F,
    z: F,
}

impl<F: Flt> OnePoleLPF<F> {
    pub fn new() -> Self {
        Self {
            israte: f::<F>(1.0) / f(44100.0),
            a: f::<F>(0.0),
            b: f::<F>(0.0),
            freq: f::<F>(1000.0),
            z: f::<F>(0.0),
        }
    }

    pub fn reset(&mut self) {
        self.z = f(0.0);
    }

    #[inline]
    fn recalc(&mut self) {
        self.b = (f::<F>(-1.0) * F::TAU() * self.freq * self.israte).exp();
        self.a = f::<F>(1.0) - self.b;
    }

    #[inline]
    pub fn set_sample_rate(&mut self, srate: F) {
        self.israte = f::<F>(1.0) / srate;
        self.recalc();
    }

    #[inline]
    pub fn set_freq(&mut self, freq: F) {
        if freq != self.freq {
            self.freq = freq;
            self.recalc();
        }
    }

    #[inline]
    pub fn process(&mut self, input: F) -> F {
        self.z = self.a * input + self.z * self.b;
        self.z
    }
}

// Fixed one pole with setable pole and gain.
// Implementation taken from tubonitaub / alec-deason
// from https://github.com/alec-deason/virtual_modular/blob/4025f1ef343c2eb9cd74eac07b5350c1e7ec9c09/src/simd_graph.rs#L4292
// under MIT License
#[derive(Debug, Copy, Clone, Default)]
pub struct FixedOnePole {
    b0: f32,
    a1: f32,
    y1: f32,
    gain: f32,
}

impl FixedOnePole {
    pub fn new(pole: f32, gain: f32) -> Self {
        let b0 = if pole > 0.0 { 1.0 - pole } else { 1.0 + pole };

        Self { b0, a1: -pole, y1: 0.0, gain }
    }

    pub fn reset(&mut self) {
        self.y1 = 0.0;
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * self.gain * input - self.a1 * self.y1;
        self.y1 = output;
        output
    }
}

// one pole hp from valley rack free:
// https://github.com/ValleyAudio/ValleyRackFree/blob/v1.0/src/Common/DSP/OnePoleFilters.cpp
#[inline]
/// Process a very simple one pole 6dB high pass filter.
/// Useful in various applications.
///
/// * `input`  - Input sample
/// * `freq`   - Frequency between 1.0 and 22000.0Hz
/// * `israte` - 1.0 / samplerate
/// * `z`      - The first internal buffer of the filter.
/// * `y`      - The second internal buffer of the filter.
///
///```
///    use synfx_dsp::*;
///
///    let samples  = vec![0.0; 44100];
///    let mut z    = 0.0;
///    let mut y    = 0.0;
///    let mut freq = 1000.0;
///
///    for s in samples.iter() {
///        let s_out =
///            process_1pole_highpass(*s, freq, 1.0 / 44100.0, &mut z, &mut y);
///        // ... do something with the result here.
///    }
///```
pub fn process_1pole_highpass(input: f32, freq: f32, israte: f32, z: &mut f32, y: &mut f32) -> f32 {
    let b = (-std::f32::consts::TAU * freq * israte).exp();
    let a = (1.0 + b) / 2.0;

    let v = a * input - a * *z + b * *y;
    *y = v;
    *z = input;
    v
}

#[derive(Debug, Clone, Copy, Default)]
pub struct OnePoleHPF<F: Flt> {
    israte: F,
    a: F,
    b: F,
    freq: F,
    z: F,
    y: F,
}

impl<F: Flt> OnePoleHPF<F> {
    pub fn new() -> Self {
        Self {
            israte: f(1.0 / 44100.0),
            a: f(0.0),
            b: f(0.0),
            freq: f(1000.0),
            z: f(0.0),
            y: f(0.0),
        }
    }

    pub fn reset(&mut self) {
        self.z = f(0.0);
        self.y = f(0.0);
    }

    #[inline]
    fn recalc(&mut self) {
        self.b = (f::<F>(-1.0) * F::TAU() * self.freq * self.israte).exp();
        self.a = (f::<F>(1.0) + self.b) / f(2.0);
    }

    pub fn set_sample_rate(&mut self, srate: F) {
        self.israte = f::<F>(1.0) / srate;
        self.recalc();
    }

    #[inline]
    pub fn set_freq(&mut self, freq: F) {
        if freq != self.freq {
            self.freq = freq;
            self.recalc();
        }
    }

    #[inline]
    pub fn process(&mut self, input: F) -> F {
        let v = self.a * input - self.a * self.z + self.b * self.y;

        self.y = v;
        self.z = input;

        v
    }
}

// one pole from:
// http://www.willpirkle.com/Downloads/AN-4VirtualAnalogFilters.pdf
// (page 5)
#[inline]
/// Process a very simple one pole 6dB low pass filter in TPT form.
/// Useful in various applications, from usage in a synthesizer to
/// damping stuff in a reverb/delay.
///
/// * `input`  - Input sample
/// * `freq`   - Frequency between 1.0 and 22000.0Hz
/// * `israte` - 1.0 / samplerate
/// * `z`      - The internal one sample buffer of the filter.
///
///```
///    use synfx_dsp::*;
///
///    let samples  = vec![0.0; 44100];
///    let mut z    = 0.0;
///    let mut freq = 1000.0;
///
///    for s in samples.iter() {
///        let s_out =
///            process_1pole_tpt_highpass(*s, freq, 1.0 / 44100.0, &mut z);
///        // ... do something with the result here.
///    }
///```
pub fn process_1pole_tpt_lowpass(input: f32, freq: f32, israte: f32, z: &mut f32) -> f32 {
    let g = (std::f32::consts::PI * freq * israte).tan();
    let a = g / (1.0 + g);

    let v1 = a * (input - *z);
    let v2 = v1 + *z;
    *z = v2 + v1;

    // let (m0, m1) = (0.0, 1.0);
    // (m0 * input + m1 * v2) as f32);
    v2
}

// one pole from:
// http://www.willpirkle.com/Downloads/AN-4VirtualAnalogFilters.pdf
// (page 5)
#[inline]
/// Process a very simple one pole 6dB high pass filter in TPT form.
/// Useful in various applications.
///
/// * `input`  - Input sample
/// * `freq`   - Frequency between 1.0 and 22000.0Hz
/// * `israte` - 1.0 / samplerate
/// * `z`      - The internal one sample buffer of the filter.
///
///```
///    use synfx_dsp::*;
///
///    let samples  = vec![0.0; 44100];
///    let mut z    = 0.0;
///    let mut freq = 1000.0;
///
///    for s in samples.iter() {
///        let s_out =
///            process_1pole_tpt_lowpass(*s, freq, 1.0 / 44100.0, &mut z);
///        // ... do something with the result here.
///    }
///```
pub fn process_1pole_tpt_highpass(input: f32, freq: f32, israte: f32, z: &mut f32) -> f32 {
    let g = (std::f32::consts::PI * freq * israte).tan();
    let a1 = g / (1.0 + g);

    let v1 = a1 * (input - *z);
    let v2 = v1 + *z;
    *z = v2 + v1;

    input - v2
}

/// The internal oversampling factor of [process_hal_chamberlin_svf].
const FILTER_OVERSAMPLE_HAL_CHAMBERLIN: usize = 2;
// Hal Chamberlin's State Variable (12dB/oct) filter
// https://www.earlevel.com/main/2003/03/02/the-digital-state-variable-filter/
// Inspired by SynthV1 by Rui Nuno Capela, under the terms of
// GPLv2 or any later:
/// Process a HAL Chamberlin filter with two delays/state variables that is 12dB.
/// The filter does internal oversampling with very simple decimation to
/// rise the stability for cutoff frequency up to 16kHz.
///
/// * `input` - Input sample.
/// * `freq` - Frequency in Hz. Please keep it inside 0.0 to 16000.0 Hz!
/// otherwise the filter becomes unstable.
/// * `res`  - Resonance from 0.0 to 0.99. Resonance of 1.0 is not recommended,
/// as the filter will then oscillate itself out of control.
/// * `israte` - 1.0 divided by the sampling rate (eg. 1.0 / 44100.0).
/// * `band` - First state variable, containing the band pass result
/// after processing.
/// * `low` - Second state variable, containing the low pass result
/// after processing.
///
/// Returned are the results of the high and notch filter.
///
///```
///    use synfx_dsp::*;
///
///    let samples  = vec![0.0; 44100];
///    let mut band = 0.0;
///    let mut low  = 0.0;
///    let mut freq = 1000.0;
///
///    for s in samples.iter() {
///        let (high, notch) =
///            process_hal_chamberlin_svf(
///                *s, freq, 0.5, 1.0 / 44100.0, &mut band, &mut low);
///        // ... do something with the result here.
///    }
///```
#[inline]
pub fn process_hal_chamberlin_svf(
    input: f32,
    freq: f32,
    res: f32,
    israte: f32,
    band: &mut f32,
    low: &mut f32,
) -> (f32, f32) {
    let q = 1.0 - res;
    let cutoff = 2.0 * (std::f32::consts::PI * freq * 0.5 * israte).sin();

    let mut high = 0.0;
    let mut notch = 0.0;

    for _ in 0..FILTER_OVERSAMPLE_HAL_CHAMBERLIN {
        *low += cutoff * *band;
        high = input - *low - q * *band;
        *band += cutoff * high;
        notch = high + *low;
    }

    //d// println!("q={:4.2} cut={:8.3} freq={:8.1} LP={:8.3} HP={:8.3} BP={:8.3} N={:8.3}",
    //d//     q, cutoff, freq, *low, high, *band, notch);

    (high, notch)
}

/// This function processes a Simper SVF with 12dB. It's a much newer algorithm
/// for filtering and provides easy to calculate multiple outputs.
///
/// * `input` - Input sample.
/// * `freq` - Frequency in Hz.
/// otherwise the filter becomes unstable.
/// * `res`  - Resonance from 0.0 to 0.99. Resonance of 1.0 is not recommended,
/// as the filter will then oscillate itself out of control.
/// * `israte` - 1.0 divided by the sampling rate (eg. 1.0 / 44100.0).
/// * `band` - First state variable, containing the band pass result
/// after processing.
/// * `low` - Second state variable, containing the low pass result
/// after processing.
///
/// This function returns the low pass, band pass and high pass signal.
/// For a notch or peak filter signal, please consult the following example:
///
///```
///    use synfx_dsp::*;
///
///    let samples   = vec![0.0; 44100];
///    let mut ic1eq = 0.0;
///    let mut ic2eq = 0.0;
///    let mut freq  = 1000.0;
///
///    for s in samples.iter() {
///        let (low, band, high) =
///            process_simper_svf(
///                *s, freq, 0.5, 1.0 / 44100.0, &mut ic1eq, &mut ic2eq);
///
///        // You can easily calculate the notch and peak results too:
///        let notch = low + high;
///        let peak  = low - high;
///        // ... do something with the result here.
///    }
///```
// Simper SVF implemented from
// https://cytomic.com/files/dsp/SvfLinearTrapezoidalSin.pdf
// Big thanks go to Andrew Simper @ Cytomic for developing and publishing
// the paper.
#[inline]
pub fn process_simper_svf(
    input: f32,
    freq: f32,
    res: f32,
    israte: f32,
    ic1eq: &mut f32,
    ic2eq: &mut f32,
) -> (f32, f32, f32) {
    // XXX: the 1.989 were tuned by hand, so the resonance is more audible.
    let k = 2f32 - (1.989f32 * res);
    let w = std::f32::consts::PI * freq * israte;

    let s1 = w.sin();
    let s2 = (2.0 * w).sin();
    let nrm = 1.0 / (2.0 + k * s2);

    let g0 = s2 * nrm;
    let g1 = (-2.0 * s1 * s1 - k * s2) * nrm;
    let g2 = (2.0 * s1 * s1) * nrm;

    let t0 = input - *ic2eq;
    let t1 = g0 * t0 + g1 * *ic1eq;
    let t2 = g2 * t0 + g0 * *ic1eq;

    let v1 = t1 + *ic1eq;
    let v2 = t2 + *ic2eq;

    *ic1eq += 2.0 * t1;
    *ic2eq += 2.0 * t2;

    // low   = v2
    // band  = v1
    // high  = input - k * v1 - v2
    // notch = low + high            = input - k * v1
    // peak  = low - high            = 2 * v2 - input + k * v1
    // all   = low + high - k * band = input - 2 * k * v1

    (v2, v1, input - k * v1 - v2)
}

/// This function implements a simple Stilson/Moog low pass filter with 24dB.
/// It provides only a low pass output.
///
/// * `input` - Input sample.
/// * `freq` - Frequency in Hz.
/// otherwise the filter becomes unstable.
/// * `res`  - Resonance from 0.0 to 0.99. Resonance of 1.0 is not recommended,
/// as the filter will then oscillate itself out of control.
/// * `israte` - 1.0 divided by the sampling rate (`1.0 / 44100.0`).
/// * `b0` to `b3` - Internal values used for filtering.
/// * `delay` - A buffer holding other delayed samples.
///
///```
///    use synfx_dsp::*;
///
///    let samples  = vec![0.0; 44100];
///    let mut b0   = 0.0;
///    let mut b1   = 0.0;
///    let mut b2   = 0.0;
///    let mut b3   = 0.0;
///    let mut delay = [0.0; 4];
///    let mut freq = 1000.0;
///
///    for s in samples.iter() {
///        let low =
///            process_stilson_moog(
///                *s, freq, 0.5, 1.0 / 44100.0,
///                &mut b0, &mut b1, &mut b2, &mut b3,
///                &mut delay);
///
///        // ... do something with the result here.
///    }
///```
// Stilson/Moog implementation partly translated from here:
// https://github.com/ddiakopoulos/MoogLadders/blob/master/src/MusicDSPModel.h
// without any copyright as found on musicdsp.org
// (http://www.musicdsp.org/showone.php?id=24).
//
// It's also found on MusicDSP and has probably no proper license anyways.
// See also: https://github.com/ddiakopoulos/MoogLadders
// and https://github.com/rncbc/synthv1/blob/master/src/synthv1_filter.h#L103
// and https://github.com/ddiakopoulos/MoogLadders/blob/master/src/MusicDSPModel.h
#[inline]
pub fn process_stilson_moog(
    input: f32,
    freq: f32,
    res: f32,
    israte: f32,
    b0: &mut f32,
    b1: &mut f32,
    b2: &mut f32,
    b3: &mut f32,
    delay: &mut [f32; 4],
) -> f32 {
    let cutoff = 2.0 * freq * israte;

    let p = cutoff * (1.8 - 0.8 * cutoff);
    let k = 2.0 * (cutoff * std::f32::consts::PI * 0.5).sin() - 1.0;

    let t1 = (1.0 - p) * 1.386249;
    let t2 = 12.0 + t1 * t1;

    let res = res * (t2 + 6.0 * t1) / (t2 - 6.0 * t1);

    let x = input - res * *b3;

    // Four cascaded one-pole filters (bilinear transform)
    *b0 = x * p + delay[0] * p - k * *b0;
    *b1 = *b0 * p + delay[1] * p - k * *b1;
    *b2 = *b1 * p + delay[2] * p - k * *b2;
    *b3 = *b2 * p + delay[3] * p - k * *b3;

    // Clipping band-limited sigmoid
    *b3 -= (*b3 * *b3 * *b3) * 0.166667;

    delay[0] = x;
    delay[1] = *b0;
    delay[2] = *b1;
    delay[3] = *b2;

    *b3
}

// translated from Odin 2 Synthesizer Plugin
// Copyright (C) 2020 TheWaveWarden
// under GPLv3 or any later
#[derive(Debug, Clone, Copy)]
pub struct DCBlockFilter<F: Flt> {
    xm1: F,
    ym1: F,
    r: F,
}

impl<F: Flt> DCBlockFilter<F> {
    pub fn new() -> Self {
        Self { xm1: f(0.0), ym1: f(0.0), r: f(0.995) }
    }

    pub fn reset(&mut self) {
        self.xm1 = f(0.0);
        self.ym1 = f(0.0);
    }

    pub fn set_sample_rate(&mut self, srate: F) {
        self.r = f(0.995);
        if srate > f(90000.0) {
            self.r = f(0.9965);
        } else if srate > f(120000.0) {
            self.r = f(0.997);
        }
    }

    pub fn next(&mut self, input: F) -> F {
        let y = input - self.xm1 + self.r * self.ym1;
        self.xm1 = input;
        self.ym1 = y;
        y as F
    }
}

