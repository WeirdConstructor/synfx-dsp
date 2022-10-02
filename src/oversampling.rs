// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! Oversampling related utilities, such as an up/downsampling filter.

use crate::{Biquad, BiquadCoefs};
use std::simd::f32x4;

// Loosely adapted from https://github.com/VCVRack/Befaco/blob/v1/src/ChowDSP.hpp
// Copyright (c) 2019-2020 Andrew Belt and Befaco contributors
// Under GPLv-3.0-or-later
//
// Which was originally taken from https://github.com/jatinchowdhury18/ChowDSP-VCV/blob/master/src/shared/AAFilter.hpp
// Copyright (c) 2020 jatinchowdhury18
/// Implements oversampling with a ratio of N and a 4 times cascade
/// of Butterworth lowpass filters (~48dB?).
#[derive(Debug, Copy, Clone)]
pub struct Oversampling<const N: usize> {
    filters: [Biquad; 4],
    buffer: [f32; N],
}

impl<const N: usize> Oversampling<N> {
    pub fn new() -> Self {
        let mut this = Self { filters: [Biquad::new(); 4], buffer: [0.0; N] };

        this.set_sample_rate(44100.0);

        this
    }

    pub fn reset(&mut self) {
        self.buffer = [0.0; N];
        for filt in &mut self.filters {
            filt.reset();
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        let cutoff = 0.98 * (0.5 * srate);

        let ovr_srate = (N as f32) * srate;
        let filters_len = self.filters.len();

        for (i, filt) in self.filters.iter_mut().enumerate() {
            let q = BiquadCoefs::calc_cascaded_butter_q(2 * 4, filters_len - i);

            filt.set_coefs(BiquadCoefs::lowpass(ovr_srate, q, cutoff));
        }
    }

    #[inline]
    pub fn upsample(&mut self, v: f32) {
        self.buffer.fill(0.0);
        self.buffer[0] = (N as f32) * v;

        for s in &mut self.buffer {
            for filt in &mut self.filters {
                *s = filt.tick(*s);
            }
        }
    }

    #[inline]
    pub fn resample_buffer(&mut self) -> &mut [f32; N] {
        &mut self.buffer
    }

    #[inline]
    pub fn downsample(&mut self) -> f32 {
        let mut ret = 0.0;
        for s in &mut self.buffer {
            ret = *s;
            for filt in &mut self.filters {
                ret = filt.tick(ret);
            }
        }

        ret
    }
}

// Taken from va-filter by Fredemus aka Frederik Halkjær aka RocketPhysician
// https://github.com/Fredemus/va-filter
// Under License GPL-3.0-or-later
//
// below is a polyphase iir halfband filter (cutoff at fs/4) consisting of cascades of allpasses
// translated from the freely available source code at https://www.musicdsp.org/en/latest/Filters/39-polyphase-filters.html
// no changes to the algorithm, just some rust-ifying and simple simding for independent samples

#[derive(Copy, Clone)]
struct Allpass {
    a: f32x4,
    x0: f32x4,
    x1: f32x4,
    x2: f32x4,

    y0: f32x4,
    y1: f32x4,
    y2: f32x4,
}

impl Default for Allpass {
    fn default() -> Allpass {
        Allpass {
            a: f32x4::splat(0.),

            x0: f32x4::splat(0.),
            x1: f32x4::splat(0.),
            x2: f32x4::splat(0.),

            y0: f32x4::splat(0.),
            y1: f32x4::splat(0.),
            y2: f32x4::splat(0.),
        }
    }
}
impl Allpass {
    fn process(&mut self, input: f32x4) -> f32x4 {
        //shuffle inputs
        self.x2 = self.x1;
        self.x1 = self.x0;
        self.x0 = input;

        //shuffle outputs
        self.y2 = self.y1;
        self.y1 = self.y0;

        //allpass filter 1
        let output = self.x2 + ((input - self.y2) * self.a);

        self.y0 = output;
        output
    }
}
#[derive(Copy, Clone)]
struct AllpassCascade {
    allpasses: [Allpass; 6],
    num_filters: usize,
}

impl AllpassCascade {
    fn process(&mut self, input: f32x4) -> f32x4 {
        let mut output = input;
        for i in 0..self.num_filters {
            output = self.allpasses[i].process(output)
        }
        output
    }
}

/// This is a polyphase iir halfband filter (cutoff at fs/4) consisting of cascades of allpasses.
/// translated from the freely available source code at <https://www.musicdsp.org/en/latest/Filters/39-polyphase-filters.html>
///
/// Usage:
///```
/// #![feature(portable_simd)]
/// use std::simd::f32x4;
///
/// use synfx_dsp::PolyIIRHalfbandFilter;
///
/// struct MyNiceDistort {
///     upsampler: PolyIIRHalfbandFilter,
///     downsampler: PolyIIRHalfbandFilter,
/// }
///
/// impl MyNiceDistort {
///     fn new() -> Self {
///         Self {
///             upsampler: PolyIIRHalfbandFilter::new(8, true),
///             downsampler: PolyIIRHalfbandFilter::new(8, true),
///         }
///     }
///
///     fn process(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
///         let frame = f32x4::from_array([in_l, in_r, 0.0, 0.0]);
///         // Zero stuffing:
///         let input = [frame, f32x4::splat(0.)];
///         // Prepare the output:
///         let mut output = f32x4::splat(0.);
///         for i in 0..2 {
///             // Upsampling:
///             let frame = self.upsampler.process(f32x4::splat(2.) * input[i]);
///
///             // Apply some non linear stuff:
///             let out = synfx_dsp::tanh_levien(frame * f32x4::splat(10.0));
///
///             // Downsampling:
///             output = self.downsampler.process(out);
///         }
///
///         let output = output.as_array();
///         (output[0], output[1])
///     }
/// }
///```
#[derive(Copy, Clone)]
pub struct PolyIIRHalfbandFilter {
    filter_a: AllpassCascade,
    filter_b: AllpassCascade,
    old_out: f32x4,
}

impl PolyIIRHalfbandFilter {
    /// Create a new [PolyIIRHalfbandFilter] with the given order.
    /// - _order_ can be 2, 4, 6, 8, 10 or 12
    /// - if _steep_ is `true`, it gives rejection of 69dB at order=8. Transition band is 0.01.
    ///   if _steep_ is `false`, it gives rejection of 106dB at order=8. Transition band is 0.05.
    pub fn new(order: usize, steep: bool) -> PolyIIRHalfbandFilter {
        let a_coefficients: Vec<f32>;
        let b_coefficients: Vec<f32>;

        if steep {
            //rejection=104dB, transition band=0.01
            if order == 12 {
                a_coefficients = vec![
                    0.036681502163648017,
                    0.2746317593794541,
                    0.56109896978791948,
                    0.769741833862266,
                    0.8922608180038789,
                    0.962094548378084,
                ];

                b_coefficients = vec![
                    0.13654762463195771,
                    0.42313861743656667,
                    0.6775400499741616,
                    0.839889624849638,
                    0.9315419599631839,
                    0.9878163707328971,
                ];
            }
            //rejection=86dB, transition band=0.01
            else if order == 10 {
                a_coefficients = vec![
                    0.051457617441190984,
                    0.35978656070567017,
                    0.6725475931034693,
                    0.8590884928249939,
                    0.9540209867860787,
                ];

                b_coefficients = vec![
                    0.18621906251989334,
                    0.529951372847964,
                    0.7810257527489514,
                    0.9141815687605308,
                    0.985475023014907,
                ];
            }
            //rejection=69dB, transition band=0.01
            else if order == 8 {
                a_coefficients = vec![
                    0.07711507983241622,
                    0.4820706250610472,
                    0.7968204713315797,
                    0.9412514277740471,
                ];

                b_coefficients = vec![
                    0.2659685265210946,
                    0.6651041532634957,
                    0.8841015085506159,
                    0.9820054141886075,
                ];
            }
            //rejection=51dB, transition band=0.01
            else if order == 6 {
                a_coefficients = vec![0.1271414136264853, 0.6528245886369117, 0.9176942834328115];

                b_coefficients = vec![0.40056789819445626, 0.8204163891923343, 0.9763114515836773];
            }
            //rejection=53dB,transition band=0.05
            else if order == 4 {
                a_coefficients = vec![0.12073211751675449, 0.6632020224193995];

                b_coefficients = vec![0.3903621872345006, 0.890786832653497];
            }
            //order=2, rejection=36dB, transition band=0.1
            else {
                a_coefficients = vec![0.23647102099689224];
                b_coefficients = vec![0.7145421497126001];
            }
        }
        //softer slopes, more attenuation and less stopband ripple
        else {
            //rejection=150dB, transition band=0.05
            if order == 12 {
                a_coefficients = vec![
                    0.01677466677723562,
                    0.13902148819717805,
                    0.3325011117394731,
                    0.53766105314488,
                    0.7214184024215805,
                    0.8821858402078155,
                ];
                b_coefficients = vec![
                    0.06501319274445962,
                    0.23094129990840923,
                    0.4364942348420355,
                    0.6329609551399348, //0.06329609551399348
                    0.80378086794111226,
                    0.9599687404800694,
                ];
            }
            //rejection=133dB, transition band=0.05
            else if order == 10 {
                a_coefficients = vec![
                    0.02366831419883467,
                    0.18989476227180174,
                    0.43157318062118555,
                    0.6632020224193995,
                    0.860015542499582,
                ];
                b_coefficients = vec![
                    0.09056555904993387,
                    0.3078575723749043,
                    0.5516782402507934,
                    0.7652146863779808,
                    0.95247728378667541,
                ];
            }
            //rejection=106dB, transition band=0.05
            else if order == 8 {
                a_coefficients = vec![
                    0.03583278843106211,
                    0.2720401433964576,
                    0.5720571972357003,
                    0.827124761997324,
                ];

                b_coefficients = vec![
                    0.1340901419430669,
                    0.4243248712718685,
                    0.7062921421386394,
                    0.9415030941737551,
                ];
            }
            //rejection=80dB, transition band=0.05
            else if order == 6 {
                a_coefficients = vec![0.06029739095712437, 0.4125907203610563, 0.7727156537429234];

                b_coefficients = vec![0.21597144456092948, 0.6043586264658363, 0.9238861386532906];
            }
            //rejection=70dB,transition band=0.1
            else if order == 4 {
                a_coefficients = vec![0.07986642623635751, 0.5453536510711322];

                b_coefficients = vec![0.28382934487410993, 0.8344118914807379];
            }
            //order=2, rejection=36dB, transition band=0.1
            else {
                a_coefficients = vec![0.23647102099689224];
                b_coefficients = vec![0.7145421497126001];
            }
        }
        let mut allpasses_a = [Allpass::default(); 6];
        for i in 0..order / 2 {
            allpasses_a[i].a = f32x4::splat(a_coefficients[i]);
        }
        let filter_a = AllpassCascade { num_filters: order / 2, allpasses: allpasses_a };
        let mut allpasses_b = [Allpass::default(); 6];
        for i in 0..order / 2 {
            allpasses_b[i].a = f32x4::splat(b_coefficients[i]);
        }
        let filter_b = AllpassCascade { num_filters: order / 2, allpasses: allpasses_b };
        PolyIIRHalfbandFilter { filter_a, filter_b, old_out: f32x4::splat(0.) }
    }

    pub fn process(&mut self, input: f32x4) -> f32x4 {
        let output = (self.filter_a.process(input) + self.old_out) * f32x4::splat(0.5);
        self.old_out = self.filter_b.process(input);
        output
    }
}

impl Default for PolyIIRHalfbandFilter {
    fn default() -> PolyIIRHalfbandFilter {
        let a_coefficients = vec![
            0.01677466677723562,
            0.13902148819717805,
            0.3325011117394731,
            0.53766105314488,
            0.7214184024215805,
            0.8821858402078155,
        ];

        let b_coefficients = vec![
            0.06501319274445962,
            0.23094129990840923,
            0.4364942348420355,
            0.6329609551399348, //0.06329609551399348
            0.80378086794111226,
            0.9599687404800694,
        ];
        let mut allpasses_a = [Allpass::default(); 6];
        for i in 0..12 / 2 {
            allpasses_a[i].a = f32x4::splat(a_coefficients[i]);
        }
        let filter_a = AllpassCascade { num_filters: 12 / 2, allpasses: allpasses_a };
        let mut allpasses_b = [Allpass::default(); 6];
        for i in 0..12 / 2 {
            allpasses_b[i].a = f32x4::splat(b_coefficients[i]);
        }
        let filter_b = AllpassCascade { num_filters: 12 / 2, allpasses: allpasses_b };
        PolyIIRHalfbandFilter { filter_a, filter_b, old_out: f32x4::splat(0.0) }
    }
}
