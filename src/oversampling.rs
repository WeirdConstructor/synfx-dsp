// Copyright (c) 2021 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::Biquad;

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
