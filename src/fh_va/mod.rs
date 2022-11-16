// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! This module contains the VA filter code of Fredemus' aka Frederik Halkjær aka RocketPhysician.
/// It's awesome for driven filters with non-linearities. I recommend using
/// [crate::oversampling::PolyIIRHalfbandFilter] oversampling with it.
///
/// VA filter implementation by Frederik Halkjær,
/// copied under GPL-3.0-or-later from <https://github.com/Fredemus/va-filter>
mod ladder;
mod solver;
use solver::DKSolver;

mod sallen_key;
mod svf;

pub use ladder::LadderFilter;
pub use sallen_key::SallenKey;
pub use svf::Svf;

/// The SVF filter mode (LP, HP, BP1, Notch, BP2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvfMode {
    LP,
    HP,
    BP1,
    Notch,
    BP2,
}

#[derive(Debug, Clone)]
/// Filter parameters for the filters [crate::fh_va::Svf], [crate::fh_va::SallenKey] and [crate::fh_va::LadderFilter].
pub struct FilterParams {
    /// Cutoff frequency 5.0 Hz to 20 kHz.
    pub cutoff: f32,
    /// Resonance, values between 0.0-1.0, default: 0.5
    pub res: f32,
    /// Filter drive, values between 1.0 and 15.8490 (gain to dB)
    pub drive: f32,

    /// The SVF filter mode.
    pub mode: SvfMode,
    /// The Ladder filter mode.
    pub ladder_mode: LadderMode,

    /// Calculated by the [FilterParams::set_frequency] function.
    pub g: f32,
    /// Use the [FilterParams::set_sample_rate] function to update this.
    pub sample_rate: f32,
    /// Resistance based internal parameter, set by [FilterParams::set_resonance].
    pub zeta: f32,
    /// Resistance based internal parameter, set by [FilterParams::set_resonance].
    pub k_ladder: f32,
}

impl FilterParams {
    pub fn new() -> Self {
        let mut this = Self {
            cutoff: 440.0,
            res: 0.5,
            drive: 1.0,

            mode: SvfMode::LP,
            ladder_mode: LadderMode::LP6,

            g: 0.0,
            sample_rate: 0.0,
            zeta: 0.0,
            k_ladder: 0.0,
        };
        this.set_sample_rate(44100.0);
        this.set_resonance(0.5);
        this.set_frequency(440.0);
        this
    }

    #[inline]
    pub fn set_resonance(&mut self, res: f32) {
        self.res = res;
        self.zeta = 5. - 5.0 * res;
        //        self.k_ladder = res.powi(2) * 3.8 - 0.2;
        self.k_ladder = res.powi(2) * 4.5 - 0.2;
    }

    #[inline]
    pub fn set_frequency(&mut self, freq: f32) {
        self.cutoff = freq;
        self.g = (std::f32::consts::PI * freq / self.sample_rate).tan();
    }

    #[inline]
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.set_resonance(self.res);
        self.set_frequency(self.cutoff);
    }
}

/// The Ladder mode, You can choose between low pass, high pass, band pass and notch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LadderMode {
    LP6,
    LP12,
    LP18,
    LP24,
    HP6,
    HP12,
    HP18,
    HP24,
    BP12,
    BP24,
    N12,
}
impl std::fmt::Display for LadderMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LadderMode::LP6 => write!(f, "LP6"),
            LadderMode::LP12 => write!(f, "LP12"),
            LadderMode::LP18 => write!(f, "LP18"),
            LadderMode::LP24 => write!(f, "LP24"),
            LadderMode::HP6 => write!(f, "HP6"),
            LadderMode::HP12 => write!(f, "HP12"),
            LadderMode::HP18 => write!(f, "HP18"),
            LadderMode::HP24 => write!(f, "HP24"),
            LadderMode::BP12 => write!(f, "BP12"),
            LadderMode::BP24 => write!(f, "BP24"),
            LadderMode::N12 => write!(f, "N12"),
        }
    }
}

pub fn get_ladder_mix(mode: LadderMode) -> [f32; 5] {
    let mix;
    match mode {
        LadderMode::LP6 => {
            mix = [0., -1., 0., -0., 0.];
        }
        LadderMode::LP12 => {
            mix = [0., -0., 1., -0., 0.];
        }
        LadderMode::LP18 => {
            mix = [0., -0., 0., -1., 0.];
        }
        LadderMode::LP24 => {
            mix = [0., -0., 0., -0., 1.];
        }
        LadderMode::HP6 => {
            mix = [1., -1., 0., -0., 0.];
        }
        LadderMode::HP12 => {
            mix = [1., -2., 1., -0., 0.];
        }
        LadderMode::HP18 => {
            mix = [1., -3., 3., -1., 0.];
        }
        LadderMode::HP24 => {
            mix = [1., -4., 6., -4., 1.];
        }
        LadderMode::BP12 => {
            mix = [0., -1., 1., -0., 0.];
        }
        LadderMode::BP24 => {
            mix = [0., -0., 1., -2., 1.];
        }
        LadderMode::N12 => {
            mix = [1., -2., 2., -0., 0.];
        }
    }
    mix
}
