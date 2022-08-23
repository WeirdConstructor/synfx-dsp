// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

/*! Provides you with all the tools for building ADSR or any other kind of envelopes.

See also:

- [EnvState] which holds the state of the envelope.
- [env_hold_stage] for a hold stage piece
- [env_target_stage] for an attack/decay/release stage piece
- [env_sustain_stage] for a sustain stage piece
*/

use crate::TRIG_LOW_THRES;

/// Envelope state structure for the macros [env_hold_stage], [env_target_stage] and [env_sustain_stage].
///
///```
/// use synfx_dsp::{EnvState, env_hold_stage, env_target_stage, assert_decimated_slope_feq};
/// let mut state = EnvState::new();
/// state.set_sample_rate(48000.0);
///
/// let attack_ms = 1.0;
/// let hold_ms = 2.0;
/// let delay_ms = 2.0;
///
/// let mut env_samples = vec![];
/// for _ in 0..(((48000.0 * (attack_ms + hold_ms + delay_ms)) / 1000.0) as usize) {
///     env_target_stage!(state, 0, attack_ms, 1.0, |x| x, {
///         env_hold_stage!(state, 2, hold_ms, {
///             env_target_stage!(state, 4, delay_ms, 0.0, |x| x, {});
///         });
///     });
///     env_samples.push(state.current);
/// }
///
/// assert_decimated_slope_feq!(env_samples[0..48], 4, vec![0.02083; 100]);
///```
pub struct EnvState {
    pub srate_ms: f32,
    pub stage: u32,
    pub phase: f32,
    pub start: f32,
    pub current: f32,
}

impl EnvState {
    /// Create a new envelope state structure.
    pub fn new() -> Self {
        Self {
            srate_ms: 44100.0 / 1000.0,
            stage: 0,
            phase: 0.0,
            start: 0.0,
            current: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate_ms = srate / 1000.0;
    }
}

/// Holds the previous `state.current` value for `$time_ms`.
///
/// See also [EnvState] about the first argument `$state`.
/// `$stage_idx` is `$stage_idx + 2` after this stage is finished.
#[macro_export]
macro_rules! env_hold_stage {
    ($state: expr, $stage_idx: expr, $time_ms: expr, $else: block) => {
        if $state.stage == $stage_idx || $state.stage == ($stage_idx + 1) {
            if $state.stage == $stage_idx {
                $state.phase = 0.0;
                $state.stage += 1;
                $state.start = $state.current;
            }

            let inc = 1.0 / ($time_ms * $state.srate_ms);
            $state.phase += inc;
            if $state.phase >= 1.0 {
                $state.stage += 1;
            }
            $state.current = $state.start;
        } else $else
    };
}

/// Increases/Decreases `state.current` value until you are at `$value` within `$time_ms`.
///
/// See also [EnvState] about the first argument `$state`.
/// `$shape_fn` can be used to shape the line of this envelope stage. Use `|x| x` for a linear envelope.
/// `$stage_idx` is `$stage_idx + 2` after this stage is finished.
#[macro_export]
macro_rules! env_target_stage {
    ($state: expr, $stage_idx: expr, $time_ms: expr, $value: expr, $shape_fn: expr, $else: block) => {
        if $state.stage == $stage_idx || $state.stage == ($stage_idx + 1) {
            if $state.stage == $stage_idx {
                $state.phase = 0.0;
                $state.start = $state.current;
                $state.stage += 1;
            }

            let inc = 1.0 / ($time_ms * $state.srate_ms);
            $state.phase += inc;
            if $state.phase >= 1.0 {
                $state.stage += 1;
                $state.current = $value;
            } else {
                let phase_shped = ($shape_fn)($state.phase);
                $state.current = $state.start * (1.0 - phase_shped) + phase_shped * $value;
            }
        } else $else
    };
}

/// Holds the previous `state.current` value until `$gate` drops below [TRIG_LOW_THRES].
///
/// See also [EnvState] about the first argument `$state`.
/// `$stage_idx` is `$stage_idx + 1` after this stage is finished.
#[macro_export]
macro_rules! env_sustain_stage {
    ($state: expr, $stage_idx: expr, $sustain_value: expr, $gate: expr, $else: block) => {
        if $state.stage == $stage_idx {
            if $gate < TRIG_LOW_THRES {
                $state.stage += 1;
            }

            $state.current = $sustain_value;
        } else $else
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_decimated_slope_feq;

    #[test]
    fn check_hold_stage() {
        let mut state = EnvState::new();

        state.current = 0.6;
        for _ in 0..88 {
            env_hold_stage!(state, 0, 2.0, {});
            println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
            assert!(state.stage == 1);
            assert!(state.current > 0.5);
        }

        env_hold_stage!(state, 0, 2.0, {});
        assert!(state.stage == 2);
        assert!(state.current > 0.5);
    }

    #[test]
    fn check_target_stage() {
        let mut state = EnvState::new();

        for _ in 0..88 {
            env_target_stage!(state, 0, 2.0, 0.6, |x| x, {});
            assert!(state.stage == 1);
            println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        }

        env_target_stage!(state, 0, 2.0, 0.6, |x| x, {});
        assert!(state.stage == 2);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.current >= 0.5999);
    }

    #[test]
    fn check_very_short_target_stage() {
        let mut state = EnvState::new();

        env_target_stage!(state, 0, 0.01, 0.6, |x| x, {});
        assert!(state.stage == 2);
        assert!(state.current == 0.6);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
    }

    #[test]
    fn check_short_target_stage() {
        let mut state = EnvState::new();

        env_target_stage!(state, 0, 0.03, 0.6, |x| x, {});
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 1);
        assert!((state.current - 0.4535).abs() < 0.0001);

        env_target_stage!(state, 0, 0.03, 0.6, |x| x, {});
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 2);
        assert!(state.current == 0.6);
    }

    #[test]
    fn check_sustain_stage() {
        let mut state = EnvState::new();

        env_sustain_stage!(state, 0, 0.5, 1.0, {});
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 0);
        assert!((state.current - 0.5).abs() < 0.0001);

        env_sustain_stage!(state, 0, 0.5, 1.0, {});
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 0);
        assert!((state.current - 0.5).abs() < 0.0001);

        env_sustain_stage!(state, 0, 0.5, 0.0, {});
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 1);
        assert!((state.current - 0.5).abs() < 0.0001);
    }

    #[test]
    fn check_sustain_stage_short() {
        let mut state = EnvState::new();

        env_sustain_stage!(state, 0, 0.5, 0.0, {});
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 1);
        assert!((state.current - 0.5).abs() < 0.0001);
    }

    #[test]
    fn check_ahd_env() {
        let mut state = EnvState::new();
        state.set_sample_rate(48000.0);

        let attack_ms = 1.0;
        let hold_ms = 2.0;
        let delay_ms = 2.0;

        let mut env_samples = vec![];
        for _ in 0..(((48000.0 * (attack_ms + hold_ms + delay_ms)) / 1000.0) as usize) {
            env_target_stage!(state, 0, attack_ms, 1.0, |x| x, {
                env_hold_stage!(state, 2, hold_ms, {
                    env_target_stage!(state, 4, delay_ms, 0.0, |x| x, {});
                });
            });
            env_samples.push(state.current);
        }

        assert_decimated_slope_feq!(env_samples[0..48], 4, vec![0.02083; 100]);
        assert_decimated_slope_feq!(env_samples[48..146], 4, vec![0.0; 20]);
        assert_decimated_slope_feq!(env_samples[146..240], 4, vec![-0.01041; 40]);
    }
}
