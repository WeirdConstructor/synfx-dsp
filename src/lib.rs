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

Copyright, Licenses, Attribution, Contributions
===============================================

Here is a list of sources parts of this library copied or translated code from:

- [crate::quicker_tanh64] / [crate::quicker_tanh]
    ```text
    quickerTanh / quickerTanh64 credits to mopo synthesis library:
    Under GPLv3 or any later.
    Little IO <littleioaudio@gmail.com>
    Matt Tytel <matthewtytel@gmail.com>
    ```
- [crate::quick_tanh64] / [crate::quick_tanh]
    ```text
    quickTanh / quickTanh64 credits to mopo synthesis library:
    Under GPLv3 or any later.
    Little IO <littleioaudio@gmail.com>
    Matt Tytel <matthewtytel@gmail.com>
    ```
- [crate::tanh_approx_drive]
    ```text
    Taken from ValleyAudio
    Copyright Dale Johnson
    https://github.dev/ValleyAudio/ValleyRackFree/tree/v2.0
    Under GPLv3 license
    ```
- [crate::AtomicFloat]
    ```text
    Implementation from vst-rs
    https://github.com/RustAudio/vst-rs/blob/master/src/util/atomic_float.rs
    Under MIT License
    Copyright (c) 2015 Marko Mijalkovic
    ```
- [crate::Biquad]
    ```text
    The implementation of this Biquad Filter has been adapted from
    SamiPerttu, Copyright (c) 2020, under the MIT License.
    See also: https://github.com/SamiPerttu/fundsp/blob/master/src/filter.rs
    ```
- [crate::DattorroReverb]
    ```text
    This file contains a reverb implementation that is based
    on Jon Dattorro's 1997 reverb algorithm. It's also largely
    based on the C++ implementation from ValleyAudio / ValleyRackFree

    ValleyRackFree Copyright (C) 2020, Valley Audio Soft, Dale Johnson
    Adapted under the GPL-3.0-or-later License.

    See also: https://github.com/ValleyAudio/ValleyRackFree/blob/v1.0/src/Plateau/Dattorro.cpp
         and: https://github.com/ValleyAudio/ValleyRackFree/blob/v1.0/src/Plateau/Dattorro.hpp

    And: https://ccrma.stanford.edu/~dattorro/music.html
    And: https://ccrma.stanford.edu/~dattorro/EffectDesignPart1.pdf
    ```
- [crate::process_1pole_lowpass] / [crate::process_1pole_highpass]
    ```text
    one pole lp from valley rack free:
    https://github.com/ValleyAudio/ValleyRackFree/blob/v1.0/src/Common/DSP/OnePoleFilters.cpp
    ```
- [crate::process_1pole_tpt_lowpass] / [crate::process_1pole_tpt_highpass]
    ```text
    one pole from:
    http://www.willpirkle.com/Downloads/AN-4VirtualAnalogFilters.pdf
    (page 5)
    ```
- [crate::FixedOnePole]
    ```text
    Fixed one pole with setable pole and gain.
    Implementation taken from tubonitaub / alec-deason
    from https://github.com/alec-deason/virtual_modular/blob/4025f1ef343c2eb9cd74eac07b5350c1e7ec9c09/src/simd_graph.rs#L4292
    under MIT License
    ```
- [crate::process_hal_chamberlin_svf]
    ```text
    Hal Chamberlin's State Variable (12dB/oct) filter
    https://www.earlevel.com/main/2003/03/02/the-digital-state-variable-filter/
    Inspired by SynthV1 by Rui Nuno Capela, under the terms of
    GPLv2 or any later:
    ```
- [crate::process_simper_svf]
    ```text
    Simper SVF implemented from
    https://cytomic.com/files/dsp/SvfLinearTrapezoidalSin.pdf
    Big thanks go to Andrew Simper @ Cytomic for developing and publishing
    the paper.
    ```
- [crate::process_stilson_moog]
    ```text
    Stilson/Moog implementation partly translated from here:
    https://github.com/ddiakopoulos/MoogLadders/blob/master/src/MusicDSPModel.h
    without any copyright as found on musicdsp.org
    (http://www.musicdsp.org/showone.php?id=24).

    It's also found on MusicDSP and has probably no proper license anyways.
    See also: https://github.com/ddiakopoulos/MoogLadders
    and https://github.com/rncbc/synthv1/blob/master/src/synthv1_filter.h#L103
    and https://github.com/ddiakopoulos/MoogLadders/blob/master/src/MusicDSPModel.h
    ```
- [crate::DCBlockFilter]
    ```text
    translated from Odin 2 Synthesizer Plugin
    Copyright (C) 2020 TheWaveWarden
    under GPLv3 or any later
    ```
- [crate::cubic_interpolate]
    ```text
    Hermite interpolation, take from
    https://github.com/eric-wood/delay/blob/main/src/delay.rs#L52

    Thanks go to Eric Wood!

    For the interpolation code:
    MIT License, Copyright (c) 2021 Eric Wood
    ```
- [crate::TriSawLFO]
    ```text
    Adapted from https://github.com/ValleyAudio/ValleyRackFree/blob/v1.0/src/Common/DSP/LFO.hpp

    ValleyRackFree Copyright (C) 2020, Valley Audio Soft, Dale Johnson
    Adapted under the GPL-3.0-or-later License.
    ```
- [crate::PolyBlepOscillator]
    ```text
    PolyBLEP by Tale
    (slightly modified)
    http://www.kvraudio.com/forum/viewtopic.php?t=375517
    from http://www.martin-finke.de/blog/articles/audio-plugins-018-polyblep-oscillator/
    ```
- [crate::VPSOscillator]
    ```text
    This oscillator is based on the work "VECTOR PHASESHAPING SYNTHESIS"
    by: Jari Kleimola*, Victor Lazzarini†, Joseph Timoney†, Vesa Välimäki*
    *Aalto University School of Electrical Engineering Espoo, Finland;
    †National University of Ireland, Maynooth Ireland

    See also this PDF: http://recherche.ircam.fr/pub/dafx11/Papers/55_e.pdf
    ```
- [crate::Oversampling]
    ```text
    Loosely adapted from https://github.com/VCVRack/Befaco/blob/v1/src/ChowDSP.hpp
    Copyright (c) 2019-2020 Andrew Belt and Befaco contributors
    Under GPLv-3.0-or-later

    Which was originally taken from https://github.com/jatinchowdhury18/ChowDSP-VCV/blob/master/src/shared/AAFilter.hpp
    Copyright (c) 2020 jatinchowdhury18
    ```
- [crate::next_xoroshiro128]
    ```text
    Taken from xoroshiro128 crate under MIT License
    Implemented by Matthew Scharley (Copyright 2016)
    https://github.com/mscharley/rust-xoroshiro128
    ```
- [crate::u64_to_open01]
    ```text
    Taken from rand::distributions
    Licensed under the Apache License, Version 2.0
    Copyright 2018 Developers of the Rand project.
    ```
- [crate::SplitMix64]
    ```text
    Copyright 2018 Developers of the Rand project.

    Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
    https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
    <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
    option. This file may not be copied, modified, or distributed
    except according to those terms.
    - splitmix64 (http://xoroshiro.di.unimi.it/splitmix64.c)
    ```
- [crate::f_distort] / [crate::f_fold_distort]
    ```text
    Ported from LMMS under GPLv2
    * DspEffectLibrary.h - library with template-based inline-effects
    * Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>

    Original source seems to be musicdsp.org, Author: Bram de Jong
    see also: https://www.musicdsp.org/en/latest/Effects/41-waveshaper.html
    ```
*/

#![feature(portable_simd)]

mod approx;
mod atomic;
mod biquad;
mod dattorro;
mod delay;
mod env;
mod filters;
mod interpolation;
mod low_freq;
mod oscillators;
mod oversampling;
mod rand;
mod test;
mod trig_clock;
mod waveshapers;

pub use approx::*;
pub use atomic::*;
pub use biquad::{Biquad, BiquadCoefs};
pub use dattorro::{DattorroReverb, DattorroReverbParams};
pub use delay::*;
pub use env::*;
pub use filters::*;
pub use interpolation::*;
pub use low_freq::*;
pub use oscillators::*;
pub use oversampling::Oversampling;
pub use oversampling::PolyIIRHalfbandFilter;
pub use rand::*;
pub use test::*;
pub use trig_clock::*;
pub use waveshapers::*;

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

/// Converts gain in decibels to a factor/coeffient
///
/// ```
/// use synfx_dsp::gain_db2coef;
///
/// assert!((gain_db2coef(-6.0) - 0.501187).abs() < 0.00001);
/// assert!((gain_db2coef(-3.0) - 0.707945).abs() < 0.00001);
/// assert!((gain_db2coef(0.0) - 1.0).abs() < 0.00001);
/// assert!((gain_db2coef(6.0) - 1.99526).abs() < 0.00001);
/// ```
#[inline]
pub fn gain_db2coef(gain_db: f32) -> f32 {
    if gain_db < -89.9 {
        0.0
    } else {
        10.0_f32.powf(gain_db * 0.05)
    }
}

/// Converts a coefficient/factor to decibels
///
///```
/// use synfx_dsp::coef2gain_db;
///
/// assert!((coef2gain_db(0.501187) - -6.0).abs() < 0.0001);
/// assert!((coef2gain_db(0.707945) - -3.0).abs() < 0.0001);
/// assert!(coef2gain_db(1.0).abs() < 0.00001);
/// assert!((coef2gain_db(1.99526) - 6.0).abs() < 0.0001);
///```
#[inline]
pub fn coef2gain_db(coef: f32) -> f32 {
    if coef < 0.0000317 {
        -90.0
    } else {
        20.0 * coef.log10()
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
