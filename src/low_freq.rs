// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! Low frequency utilities for handling control signals (partially also at audio rate).

use crate::{Flt, f, fclampc};

// Adapted from https://github.com/ValleyAudio/ValleyRackFree/blob/v1.0/src/Common/DSP/LFO.hpp
//
// ValleyRackFree Copyright (C) 2020, Valley Audio Soft, Dale Johnson
// Adapted under the GPL-3.0-or-later License.
/// An LFO with a variable reverse point, which can go from reverse Saw, to Tri
/// and to Saw, depending on the reverse point.
#[derive(Debug, Clone, Copy)]
pub struct TriSawLFO<F: Flt> {
    /// The (inverse) sample rate. Eg. 1.0 / 44100.0.
    israte: F,
    /// The current oscillator phase.
    phase: F,
    /// The point from where the falling edge will be used.
    rev: F,
    /// The frequency.
    freq: F,
    /// Precomputed rise/fall rate of the LFO.
    rise_r: F,
    fall_r: F,
    /// Initial phase offset.
    init_phase: F,
}

impl<F: Flt> TriSawLFO<F> {
    pub fn new() -> Self {
        let mut this = Self {
            israte: f(1.0 / 44100.0),
            phase: f(0.0),
            rev: f(0.5),
            freq: f(1.0),
            fall_r: f(0.0),
            rise_r: f(0.0),
            init_phase: f(0.0),
        };
        this.recalc();
        this
    }

    pub fn set_phase_offs(&mut self, phase: F) {
        self.init_phase = phase;
        self.phase = phase;
    }

    #[inline]
    fn recalc(&mut self) {
        self.rev = fclampc(self.rev, 0.0001, 0.999);
        self.rise_r = f::<F>(1.0) / self.rev;
        self.fall_r = f::<F>(-1.0) / (f::<F>(1.0) - self.rev);
    }

    pub fn set_sample_rate(&mut self, srate: F) {
        self.israte = f::<F>(1.0) / (srate as F);
        self.recalc();
    }

    pub fn reset(&mut self) {
        self.phase = self.init_phase;
        self.rev = f(0.5);
    }

    #[inline]
    pub fn set(&mut self, freq: F, rev: F) {
        self.freq = freq as F;
        self.rev = rev as F;
        self.recalc();
    }

    #[inline]
    pub fn next_unipolar(&mut self) -> F {
        if self.phase >= f(1.0) {
            self.phase = self.phase - f(1.0);
        }

        let s = if self.phase < self.rev {
            self.phase * self.rise_r
        } else {
            self.phase * self.fall_r - self.fall_r
        };

        self.phase = self.phase + self.freq * self.israte;

        s
    }

    #[inline]
    pub fn next_bipolar(&mut self) -> F {
        (self.next_unipolar() * f(2.0)) - f(1.0)
    }
}

/// A slew rate limiter, with a configurable time per 1.0 increase.
#[derive(Debug, Clone, Copy)]
pub struct SlewValue<F: Flt> {
    current: F,
    slew_per_ms: F,
}

impl<F: Flt> SlewValue<F> {
    pub fn new() -> Self {
        Self { current: f(0.0), slew_per_ms: f(1000.0 / 44100.0) }
    }

    pub fn reset(&mut self) {
        self.current = f(0.0);
    }

    pub fn set_sample_rate(&mut self, srate: F) {
        self.slew_per_ms = f::<F>(1000.0) / srate;
    }

    #[inline]
    pub fn value(&self) -> F {
        self.current
    }

    /// * `slew_ms_per_1` - The time (in milliseconds) it should take
    /// to get to 1.0 from 0.0.
    #[inline]
    pub fn next(&mut self, target: F, slew_ms_per_1: F) -> F {
        // at 0.11ms, there are barely enough samples for proper slew.
        if slew_ms_per_1 < f(0.11) {
            self.current = target;
        } else {
            let max_delta = self.slew_per_ms / slew_ms_per_1;
            self.current = target.min(self.current + max_delta).max(self.current - max_delta);
        }

        self.current
    }
}

/// A ramped value changer, with a configurable time to reach the target value.
#[derive(Debug, Clone, Copy)]
pub struct RampValue<F: Flt> {
    slew_count: u64,
    current: F,
    target: F,
    inc: F,
    sr_ms: F,
}

impl<F: Flt> RampValue<F> {
    pub fn new() -> Self {
        Self {
            slew_count: 0,
            current: f(0.0),
            target: f(0.0),
            inc: f(0.0),
            sr_ms: f(44100.0 / 1000.0),
        }
    }

    pub fn reset(&mut self) {
        self.slew_count = 0;
        self.current = f(0.0);
        self.target = f(0.0);
        self.inc = f(0.0);
    }

    pub fn set_sample_rate(&mut self, srate: F) {
        self.sr_ms = srate / f(1000.0);
    }

    #[inline]
    pub fn set_target(&mut self, target: F, slew_time_ms: F) {
        self.target = target;

        // 0.02ms, thats a fraction of a sample at 44.1kHz
        if slew_time_ms < f(0.02) {
            self.current = self.target;
            self.slew_count = 0;
        } else {
            let slew_samples = slew_time_ms * self.sr_ms;
            self.slew_count = slew_samples.to_u64().unwrap_or(0);
            self.inc = (self.target - self.current) / slew_samples;
        }
    }

    #[inline]
    pub fn value(&self) -> F {
        self.current
    }

    #[inline]
    pub fn next(&mut self) -> F {
        if self.slew_count > 0 {
            self.current = self.current + self.inc;
            self.slew_count -= 1;
        } else {
            self.current = self.target;
        }

        self.current
    }
}

#[derive(Debug, Clone)]
pub struct Quantizer {
    old_mask: i64,
    lkup_tbl: [(f32, f32); 24],
    last_key: f32,
}

impl Quantizer {
    pub fn new() -> Self {
        Self { old_mask: 0xFFFF_FFFF, lkup_tbl: [(0.0, 0.0); 24], last_key: 0.0 }
    }

    #[inline]
    pub fn set_keys(&mut self, keys_mask: i64) {
        if keys_mask == self.old_mask {
            return;
        }
        self.old_mask = keys_mask;

        self.setup_lookup_table();
    }

    #[inline]
    fn setup_lookup_table(&mut self) {
        let mask = self.old_mask;
        let any_enabled = mask > 0x0;

        for i in 0..24 {
            let mut min_d_note_idx = 0;
            let mut min_dist = 1000000000;

            for note in -12..=24 {
                let dist = ((i + 1_i64) / 2 - note).abs();
                let note_idx = note.rem_euclid(12);

                // XXX: We add 9 here for the mask lookup,
                // to shift the keyboard, which starts at C!
                // And first bit in the mask is the C note. 10th is the A note.
                if any_enabled && (mask & (0x1 << ((note_idx + 9) % 12))) == 0x0 {
                    continue;
                }

                //d// println!("I={:3} NOTE={:3} (IDX={:3} => bitset {}) DIST={:3}",
                //d//     i, note, note_idx,
                //d//     if (mask & (0x1 << ((note_idx + 9) % 12))) > 0x0 { 1 } else { 0 },
                //d//     dist);

                if dist < min_dist {
                    min_d_note_idx = note;
                    min_dist = dist;
                } else {
                    break;
                }
            }

            self.lkup_tbl[i as usize] = (
                (min_d_note_idx + 9).rem_euclid(12) as f32 * (0.1 / 12.0),
                min_d_note_idx.rem_euclid(12) as f32 * (0.1 / 12.0)
                    + (if min_d_note_idx < 0 {
                        -0.1
                    } else if min_d_note_idx > 11 {
                        0.1
                    } else {
                        0.0
                    }),
            );
        }
        //d// println!("TBL: {:?}", self.lkup_tbl);
    }

    #[inline]
    pub fn last_key_pitch(&self) -> f32 {
        self.last_key
    }

    #[inline]
    pub fn process(&mut self, inp: f32) -> f32 {
        let note_num = (inp * 240.0).round() as i64;
        let octave = note_num.div_euclid(24);
        let note_idx = note_num - octave * 24;

        //        println!(
        //            "INP {:7.4} => octave={:3}, note_idx={:3} note_num={:3} inp={:9.6}",
        //            inp, octave, note_idx, note_num, inp * 240.0);
        //d// println!("TBL: {:?}", self.lkup_tbl);

        let (ui_key_pitch, note_pitch) = self.lkup_tbl[note_idx as usize % 24];
        self.last_key = ui_key_pitch;
        note_pitch + octave as f32 * 0.1
    }
}

#[derive(Debug, Clone)]
pub struct CtrlPitchQuantizer {
    /// All keys, containing the min/max octave!
    keys: Vec<f32>,
    /// Only the used keys with their pitches from the UI
    used_keys: [f32; 12],
    /// A value combination of the arguments to `update_keys`.
    input_params: u64,
    /// The number of used keys from the mask.
    mask_key_count: u16,
    /// The last key for the pitch that was returned by `process`.
    last_key: u8,
}

const QUANT_TUNE_TO_A4: f32 = (9.0 / 12.0) * 0.1;

impl CtrlPitchQuantizer {
    pub fn new() -> Self {
        Self {
            keys: vec![0.0; 12 * 10],
            used_keys: [0.0; 12],
            mask_key_count: 0,
            input_params: 0xFFFFFFFFFF,
            last_key: 0,
        }
    }

    #[inline]
    pub fn last_key_pitch(&self) -> f32 {
        self.used_keys[self.last_key as usize % (self.mask_key_count as usize)] + QUANT_TUNE_TO_A4
    }

    #[inline]
    pub fn update_keys(&mut self, mut mask: i64, min_oct: i64, max_oct: i64) {
        let inp_params = (mask as u64) | ((min_oct as u64) << 12) | ((max_oct as u64) << 20);

        if self.input_params == inp_params {
            return;
        }

        self.input_params = inp_params;

        let mut mask_count = 0;

        // set all keys, if none are set!
        if mask == 0x0 {
            mask = 0xFFFF;
        }

        for i in 0..12 {
            if mask & (0x1 << i) > 0 {
                self.used_keys[mask_count] = (i as f32 / 12.0) * 0.1 - QUANT_TUNE_TO_A4;
                mask_count += 1;
            }
        }

        self.keys.clear();

        let min_oct = min_oct as usize;
        for o in 0..min_oct {
            let o = min_oct - o;

            for i in 0..mask_count {
                self.keys.push(self.used_keys[i] - (o as f32) * 0.1);
            }
        }

        for i in 0..mask_count {
            self.keys.push(self.used_keys[i]);
        }

        let max_oct = max_oct as usize;
        for o in 1..=max_oct {
            for i in 0..mask_count {
                self.keys.push(self.used_keys[i] + (o as f32) * 0.1);
            }
        }

        self.mask_key_count = mask_count as u16;
    }

    #[inline]
    pub fn signal_to_pitch(&mut self, inp: f32) -> f32 {
        let len = self.keys.len();
        let key = (inp.clamp(0.0, 0.9999) * (len as f32)).floor();
        let key = key as usize % len;
        self.last_key = key as u8;
        self.keys[key]
    }
}

