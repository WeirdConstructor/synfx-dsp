// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! Various "voltage" controlled (usually band limited) oscillator implementations.

use crate::fast_sin;

// PolyBLEP by Tale
// (slightly modified)
// http://www.kvraudio.com/forum/viewtopic.php?t=375517
// from http://www.martin-finke.de/blog/articles/audio-plugins-018-polyblep-oscillator/
//
// default for `pw' should be 1.0, it's the pulse width
// for the square wave.
#[allow(dead_code)]
fn poly_blep_64(t: f64, dt: f64) -> f64 {
    if t < dt {
        let t = t / dt;
        2. * t - (t * t) - 1.
    } else if t > (1.0 - dt) {
        let t = (t - 1.0) / dt;
        (t * t) + 2. * t + 1.
    } else {
        0.
    }
}

fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        2. * t - (t * t) - 1.
    } else if t > (1.0 - dt) {
        let t = (t - 1.0) / dt;
        (t * t) + 2. * t + 1.
    } else {
        0.
    }
}

/// This is a band-limited oscillator based on the PolyBlep technique.
///
/// **NOTE:** You need to call [crate::init_cos_tab].
///
/// Here is a quick example on how to use it:
///
///```
/// use synfx_dsp::{PolyBlepOscillator, rand_01, init_cos_tab};
/// init_cos_tab();
///
/// // Randomize the initial phase to make cancellation on summing less
/// // likely:
/// let mut osc =
///     PolyBlepOscillator::new(rand_01() * 0.25);
///
///
/// let freq   = 440.0; // Hz
/// let israte = 1.0 / 44100.0; // Seconds per Sample
/// let pw     = 0.2;   // Pulse-Width for the next_pulse()
/// let waveform = 0;   // 0 being pulse in this example, 1 being sawtooth
///
/// let mut block_of_samples = [0.0; 128];
/// // in your process function:
/// for output_sample in block_of_samples.iter_mut() {
///    *output_sample =
///        if waveform == 1 {
///             osc.next_saw(freq, israte)
///        } else {
///             osc.next_pulse(freq, israte, pw)
///        }
/// }
///```
#[derive(Debug, Clone)]
pub struct PolyBlepOscillator {
    phase: f32,
    init_phase: f32,
    last_output: f32,
}

impl PolyBlepOscillator {
    /// Create a new instance of [PolyBlepOscillator].
    ///
    /// * `init_phase` - Initial phase of the oscillator.
    /// Range of this parameter is from 0.0 to 1.0. Passing a random
    /// value is advised for preventing phase cancellation when summing multiple
    /// oscillators.
    ///
    ///```
    /// use synfx_dsp::*;
    ///
    /// let mut osc = PolyBlepOscillator::new(rand_01() * 0.25);
    ///```
    pub fn new(init_phase: f32) -> Self {
        Self { phase: 0.0, last_output: 0.0, init_phase }
    }

    /// Reset the internal state of the oscillator as if you just called
    /// [PolyBlepOscillator::new].
    #[inline]
    pub fn reset(&mut self) {
        self.phase = self.init_phase;
        self.last_output = 0.0;
    }

    /// Creates the next sample of a sine wave.
    ///
    /// * `freq` - The frequency in Hz.
    /// * `israte` - The inverse sampling rate, or seconds per sample as in eg. `1.0 / 44100.0`.
    ///```
    /// use synfx_dsp::*;
    ///
    /// let mut osc = PolyBlepOscillator::new(rand_01() * 0.25);
    ///
    /// let freq   = 440.0; // Hz
    /// let israte = 1.0 / 44100.0; // Seconds per Sample
    ///
    /// // ...
    /// let sample = osc.next_sin(freq, israte);
    /// // ...
    ///```
    #[inline]
    pub fn next_sin(&mut self, freq: f32, israte: f32) -> f32 {
        let phase_inc = freq * israte;

        let s = fast_sin(self.phase * 2.0 * std::f32::consts::PI);

        self.phase += phase_inc;
        self.phase = self.phase.fract();

        s as f32
    }

    /// Creates the next sample of a triangle wave. Please note that the
    /// bandlimited waveform needs a few initial samples to swing in.
    ///
    /// * `freq` - The frequency in Hz.
    /// * `israte` - The inverse sampling rate, or seconds per sample as in eg. `1.0 / 44100.0`.
    ///```
    /// use synfx_dsp::*;
    ///
    /// let mut osc = PolyBlepOscillator::new(rand_01() * 0.25);
    ///
    /// let freq   = 440.0; // Hz
    /// let israte = 1.0 / 44100.0; // Seconds per Sample
    ///
    /// // ...
    /// let sample = osc.next_tri(freq, israte);
    /// // ...
    ///```
    #[inline]
    pub fn next_tri(&mut self, freq: f32, israte: f32) -> f32 {
        let phase_inc = freq * israte;

        let mut s = if self.phase < 0.5 { 1.0 } else { -1.0 };

        s += poly_blep(self.phase, phase_inc);
        s -= poly_blep((self.phase + 0.5).fract(), phase_inc);

        // leaky integrator: y[n] = A * x[n] + (1 - A) * y[n-1]
        s = phase_inc * s + (1.0 - phase_inc) * self.last_output;
        self.last_output = s;

        self.phase += phase_inc;
        self.phase = self.phase.fract();

        // the signal is a bit too weak, we need to amplify it
        // or else the volume diff between the different waveforms
        // is too big:
        s * 4.0
    }

    /// Creates the next sample of a sawtooth wave.
    ///
    /// * `freq` - The frequency in Hz.
    /// * `israte` - The inverse sampling rate, or seconds per sample as in eg. `1.0 / 44100.0`.
    ///```
    /// use synfx_dsp::*;
    ///
    /// let mut osc = PolyBlepOscillator::new(rand_01() * 0.25);
    ///
    /// let freq   = 440.0; // Hz
    /// let israte = 1.0 / 44100.0; // Seconds per Sample
    ///
    /// // ...
    /// let sample = osc.next_saw(freq, israte);
    /// // ...
    ///```
    #[inline]
    pub fn next_saw(&mut self, freq: f32, israte: f32) -> f32 {
        let phase_inc = freq * israte;

        let mut s = (2.0 * self.phase) - 1.0;
        s -= poly_blep(self.phase, phase_inc);

        self.phase += phase_inc;
        self.phase = self.phase.fract();

        s
    }

    /// Creates the next sample of a pulse wave.
    /// In comparison to [PolyBlepOscillator::next_pulse_no_dc] this
    /// version is DC compensated, so that you may add multiple different
    /// pulse oscillators for a unison effect without too big DC issues.
    ///
    /// * `freq` - The frequency in Hz.
    /// * `israte` - The inverse sampling rate, or seconds per sample as in eg. `1.0 / 44100.0`.
    /// * `pw` - The pulse width. Use the value 0.0 for a square wave.
    ///```
    /// use synfx_dsp::*;
    ///
    /// let mut osc = PolyBlepOscillator::new(rand_01() * 0.25);
    ///
    /// let freq   = 440.0; // Hz
    /// let israte = 1.0 / 44100.0; // Seconds per Sample
    /// let pw     = 0.0; // 0.0 is a square wave.
    ///
    /// // ...
    /// let sample = osc.next_pulse(freq, israte, pw);
    /// // ...
    ///```
    #[inline]
    pub fn next_pulse(&mut self, freq: f32, israte: f32, pw: f32) -> f32 {
        let phase_inc = freq * israte;

        let pw = (0.1 * pw) + ((1.0 - pw) * 0.5); // some scaling
        let dc_compensation = (0.5 - pw) * 2.0;

        let mut s = if self.phase < pw { 1.0 } else { -1.0 };

        s += poly_blep(self.phase, phase_inc);
        s -= poly_blep((self.phase + (1.0 - pw)).fract(), phase_inc);

        s += dc_compensation;

        self.phase += phase_inc;
        self.phase = self.phase.fract();

        s
    }

    /// Creates the next sample of a pulse wave.
    /// In comparison to [PolyBlepOscillator::next_pulse] this
    /// version is not DC compensated. So be careful when adding multiple
    /// of this or generally using it in an audio context.
    ///
    /// * `freq` - The frequency in Hz.
    /// * `israte` - The inverse sampling rate, or seconds per sample as in eg. `1.0 / 44100.0`.
    /// * `pw` - The pulse width. Use the value 0.0 for a square wave.
    ///```
    /// use synfx_dsp::*;
    ///
    /// let mut osc = PolyBlepOscillator::new(rand_01() * 0.25);
    ///
    /// let freq   = 440.0; // Hz
    /// let israte = 1.0 / 44100.0; // Seconds per Sample
    /// let pw     = 0.0; // 0.0 is a square wave.
    ///
    /// // ...
    /// let sample = osc.next_pulse_no_dc(freq, israte, pw);
    /// // ...
    ///```
    #[inline]
    pub fn next_pulse_no_dc(&mut self, freq: f32, israte: f32, pw: f32) -> f32 {
        let phase_inc = freq * israte;

        let pw = (0.1 * pw) + ((1.0 - pw) * 0.5); // some scaling

        let mut s = if self.phase < pw { 1.0 } else { -1.0 };

        s += poly_blep(self.phase, phase_inc);
        s -= poly_blep((self.phase + (1.0 - pw)).fract(), phase_inc);

        self.phase += phase_inc;
        self.phase = self.phase.fract();

        s
    }
}

// This oscillator is based on the work "VECTOR PHASESHAPING SYNTHESIS"
// by: Jari Kleimola*, Victor Lazzarini†, Joseph Timoney†, Vesa Välimäki*
// *Aalto University School of Electrical Engineering Espoo, Finland;
// †National University of Ireland, Maynooth Ireland
//
// See also this PDF: http://recherche.ircam.fr/pub/dafx11/Papers/55_e.pdf
/// Vector Phase Shaping Oscillator.
/// The parameters `d` and `v` control the shape of the sinus
/// wave. This leads to interesting modulation properties of those
/// control values.
///
///```
/// use synfx_dsp::*;
///
/// // Randomize the initial phase to make cancellation on summing less
/// // likely:
/// let mut osc =
///     VPSOscillator::new(rand_01() * 0.25);
///
///
/// let freq   = 440.0; // Hz
/// let israte = 1.0 / 44100.0; // Seconds per Sample
/// let d      = 0.5;   // Range: 0.0 to 1.0
/// let v      = 0.75;  // Range: 0.0 to 1.0
///
/// let mut block_of_samples = [0.0; 128];
/// // in your process function:
/// for output_sample in block_of_samples.iter_mut() {
///     // It is advised to limit the `v` value, because with certain
///     // `d` values the combination creates just a DC offset.
///     let v = VPSOscillator::limit_v(d, v);
///     *output_sample = osc.next(freq, israte, d, v);
/// }
///```
///
/// It can be beneficial to apply distortion and oversampling.
/// Especially oversampling can be important for some `d` and `v`
/// combinations, even without distortion.
///
///```
/// use synfx_dsp::{VPSOscillator, rand_01, apply_distortion};
/// use synfx_dsp::Oversampling;
///
/// let mut osc = VPSOscillator::new(rand_01() * 0.25);
/// let mut ovr : Oversampling<4> = Oversampling::new();
///
/// let freq   = 440.0; // Hz
/// let israte = 1.0 / 44100.0; // Seconds per Sample
/// let d      = 0.5;   // Range: 0.0 to 1.0
/// let v      = 0.75;  // Range: 0.0 to 1.0
///
/// let mut block_of_samples = [0.0; 128];
/// // in your process function:
/// for output_sample in block_of_samples.iter_mut() {
///     // It is advised to limit the `v` value, because with certain
///     // `d` values the combination creates just a DC offset.
///     let v = VPSOscillator::limit_v(d, v);
///
///     let overbuf = ovr.resample_buffer();
///     for b in overbuf {
///         *b = apply_distortion(osc.next(freq, israte, d, v), 0.9,  1);
///     }
///     *output_sample = ovr.downsample();
/// }
///```
#[derive(Debug, Clone)]
pub struct VPSOscillator {
    phase: f32,
    init_phase: f32,
}

impl VPSOscillator {
    /// Create a new instance of [VPSOscillator].
    ///
    /// * `init_phase` - The initial phase of the oscillator.
    pub fn new(init_phase: f32) -> Self {
        Self { phase: 0.0, init_phase }
    }

    /// Reset the phase of the oscillator to the initial phase.
    #[inline]
    pub fn reset(&mut self) {
        self.phase = self.init_phase;
    }

    #[inline]
    fn s(p: f32) -> f32 {
        -(std::f32::consts::TAU * p).cos()
    }

    #[inline]
    fn phi_vps(x: f32, v: f32, d: f32) -> f32 {
        if x < d {
            (v * x) / d
        } else {
            v + ((1.0 - v) * (x - d)) / (1.0 - d)
        }
    }

    /// This rather complicated function blends out some
    /// combinations of 'd' and 'v' that just lead to a constant DC
    /// offset. Which is not very useful in an audio oscillator
    /// context.
    ///
    /// Call this before passing `v` to [VPSOscillator::next].
    #[inline]
    pub fn limit_v(d: f32, v: f32) -> f32 {
        let delta = 0.5 - (d - 0.5).abs();
        if delta < 0.05 {
            let x = (0.05 - delta) * 19.99;
            if d < 0.5 {
                let mm = x * 0.5;
                let max = 1.0 - mm;
                if v > max && v < 1.0 {
                    max
                } else if v >= 1.0 && v < (1.0 + mm) {
                    1.0 + mm
                } else {
                    v
                }
            } else {
                if v < 1.0 {
                    v.clamp(x * 0.5, 1.0)
                } else {
                    v
                }
            }
        } else {
            v
        }
    }

    /// Creates the next sample of this oscillator.
    ///
    /// * `freq` - The frequency in Hz.
    /// * `israte` - The inverse sampling rate, or seconds per sample as in eg. `1.0 / 44100.0`.
    /// * `d` - The phase distortion parameter `d` which must be in the range `0.0` to `1.0`.
    /// * `v` - The phase distortion parameter `v` which must be in the range `0.0` to `1.0`.
    ///
    /// It is advised to limit the `v` using the [VPSOscillator::limit_v] function
    /// before calling this function. To prevent DC offsets when modulating the parameters.
    pub fn next(&mut self, freq: f32, israte: f32, d: f32, v: f32) -> f32 {
        let s = Self::s(Self::phi_vps(self.phase, v, d));

        self.phase += freq * israte;
        self.phase = self.phase.fract();

        s
    }
}

//pub struct UnisonBlep {
//    oscs: Vec<PolyBlepOscillator>,
////    dc_block: crate::filter::DCBlockFilter,
//}
//
//impl UnisonBlep {
//    pub fn new(max_unison: usize) -> Self {
//        let mut oscs = vec![];
//        let mut rng = RandGen::new();
//
//        let dis_init_phase = 0.05;
//        for i in 0..(max_unison + 1) {
//            // randomize phases so we fatten the unison, get
//            // less DC and not an amplified signal until the
//            // detune desyncs the waves.
//            // But no random phase for first, so we reduce the click
//            let init_phase =
//                if i == 0 { 0.0 } else { rng.next_open01() };
//            oscs.push(PolyBlepOscillator::new(init_phase));
//        }
//
//        Self {
//            oscs,
////            dc_block: crate::filter::DCBlockFilter::new(),
//        }
//    }
//
//    pub fn set_sample_rate(&mut self, srate: f32) {
////        self.dc_block.set_sample_rate(srate);
//        for o in self.oscs.iter_mut() {
//            o.set_sample_rate(srate);
//        }
//    }
//
//    pub fn reset(&mut self) {
////        self.dc_block.reset();
//        for o in self.oscs.iter_mut() {
//            o.reset();
//        }
//    }
//
//    pub fn next<P: OscillatorInputParams>(&mut self, params: &P) -> f32 {
//        let unison =
//            (params.unison().floor() as usize)
//            .min(self.oscs.len() - 1);
//        let detune = params.detune() as f64;
//
//        let mix = (1.0 / ((unison + 1) as f32)).sqrt();
//
//        let mut s = mix * self.oscs[0].next(params, 0.0);
//
//        for u in 0..unison {
//            let detune_factor =
//                detune * (((u / 2) + 1) as f64
//                          * if (u % 2) == 0 { 1.0 } else { -1.0 });
//            s += mix * self.oscs[u + 1].next(params, detune_factor * 0.01);
//        }
//
////        self.dc_block.next(s)
//        s
//    }
//}
