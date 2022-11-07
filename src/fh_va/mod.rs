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

/// The Ladder filter slope 6dB to 24dB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LadderSlope {
    LP6,
    LP12,
    LP18,
    LP24,
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
    /// The ladder filter slope (6dB to 24dB)
    pub slope: LadderSlope,

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
            slope: LadderSlope::LP6,

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

#[derive(Debug, PartialEq)]
pub enum LadderMode {
    Lp6,
    Lp12,
    Lp18,
    Lp24,
    Hp6,
    Hp12,
    Hp18,
    Hp24,
    Bp12,
    Bp24,
    N12,
}
impl std::fmt::Display for LadderMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LadderMode::Lp6 => write!(f, "Lp6"),
            LadderMode::Lp12 => write!(f, "Lp12"),
            LadderMode::Lp18 => write!(f, "Lp18"),
            LadderMode::Lp24 => write!(f, "Lp24"),
            LadderMode::Hp6 => write!(f, "Hp6"),
            LadderMode::Hp12 => write!(f, "Hp12"),
            LadderMode::Hp18 => write!(f, "Hp18"),
            LadderMode::Hp24 => write!(f, "Hp24"),
            LadderMode::Bp12 => write!(f, "Bp12"),
            LadderMode::Bp24 => write!(f, "Bp24"),
            LadderMode::N12 => write!(f, "N12"),
        }
    }
}
pub fn get_ladder_mix(mode: LadderMode) -> [f32; 5] {
    let mix;
    match mode {
        LadderMode::Lp6 => {
            mix = [0., -1., 0., -0., 0.];
        }
        LadderMode::Lp12 => {
            mix = [0., -0., 1., -0., 0.];
        }
        LadderMode::Lp18 => {
            mix = [0., -0., 0., -1., 0.];
        }
        LadderMode::Lp24 => {
            mix = [0., -0., 0., -0., 1.];
        }
        LadderMode::Hp6 => {
            mix = [1., -1., 0., -0., 0.];
        }
        LadderMode::Hp12 => {
            mix = [1., -2., 1., -0., 0.];
        }
        LadderMode::Hp18 => {
            mix = [1., -3., 3., -1., 0.];
        }
        LadderMode::Hp24 => {
            mix = [1., -4., 6., -4., 1.];
        }
        LadderMode::Bp12 => {
            mix = [0., -1., 1., -0., 0.];
        }
        LadderMode::Bp24 => {
            mix = [0., -0., 1., -2., 1.];
        } 
        LadderMode::N12 => {
            mix = [1., -2., 2., -0., 0.];

        },
          
    }
    mix
}