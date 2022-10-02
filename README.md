# synfx-dsp

synfx-dsp DSP real time audio synthesis, effect algorithms and utilities for Rust

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

**Requires Nightly as of 2022-10-02 due to std::simd!**

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
- [crate::PolyIIRHalfbandFilter]
    ```text
    Taken from va-filter by Fredemus aka Frederik Halkjær aka RocketPhysician
    https://github.com/Fredemus/va-filter
    Under License GPL-3.0-or-later

    originally translated from the freely available source code
    at https://www.musicdsp.org/en/latest/Filters/39-polyphase-filters.html
    ```
- [crate::fh_va::LadderFilter] / [crate::fh_va::Svf] / [crate::fh_va::SallenKey]
    ```text
    Taken from va-filter by Fredemus aka Frederik Halkjær aka RocketPhysician
    https://github.com/Fredemus/va-filter
    Under License GPL-3.0-or-later
    ```
License: GPL-3.0-or-later
