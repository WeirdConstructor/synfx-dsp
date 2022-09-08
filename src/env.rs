// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

/*! Provides you with all the tools for building ADSR or any other kind of envelopes.

See also:

- [EnvState] which holds the state of the envelope.
- [EnvRetrigAD] is a complete implementation of an attack decay envelope.
- [crate::env_hold_stage] for a hold stage piece
- [crate::env_target_stage] for an attack/decay/release stage piece
- [crate::env_sustain_stage] for a sustain stage piece
*/

use crate::sqrt4_to_pow4;
use crate::{TrigSignal, Trigger};

/// Envelope state structure for the macros [crate::env_hold_stage],
/// [crate::env_target_stage] and [crate::env_sustain_stage].
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
/// state.trigger();
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
#[derive(Debug, Clone)]
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
            stage: std::u32::MAX,
            phase: 0.0,
            start: 0.0,
            current: 0.0,
        }
    }

    #[inline]
    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate_ms = srate / 1000.0;
    }

    #[inline]
    pub fn trigger(&mut self) {
        self.stage = 0;
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        self.stage != std::u32::MAX
    }

    #[inline]
    pub fn stop_immediately(&mut self) {
        self.stage = std::u32::MAX;
    }

    pub fn reset(&mut self) {
        self.stage = std::u32::MAX;
        self.phase = 0.0;
        self.start = 0.0;
        self.current = 0.0;
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
/// This envelope part is great for a release stage. Or an fixed time attack stage.
/// See also [crate::env_target_stage_lin_time_adj].
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

            let inc = 1.0 / ($time_ms * $state.srate_ms).max(1.0);
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

/// Increases/Decreases `state.current` value until you are at `$value` within an adjusted `$time_ms`.
/// Depending on how close `state.start` is to `$value`, `$time_ms` is linearily shortened.
/// For this to work, you need to supply the supposed starting value of the envelope.
///
/// This envelope part is great for a retriggerable envelope.
///
/// See also [EnvState] about the first argument `$state`.
/// `$shape_fn` can be used to shape the line of this envelope stage. Use `|x| x` for a linear envelope.
/// `$stage_idx` is `$stage_idx + 2` after this stage is finished.
#[macro_export]
macro_rules! env_target_stage_lin_time_adj {
    ($state: expr, $stage_idx: expr, $time_ms: expr, $src_value: expr, $value: expr, $shape_fn: expr, $else: block) => {
        if $state.stage == $stage_idx || $state.stage == ($stage_idx + 1) {
            if $state.stage == $stage_idx {
                $state.phase = 0.0;
                $state.start = $state.current;
                $state.stage += 1;
            }

            let time_adj_factor = 1.0 - ($state.start - $src_value) / ($value - $src_value);
            let inc = 1.0 / (time_adj_factor * $time_ms * $state.srate_ms).max(1.0);
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

/// Holds the previous `state.current` value until `$gate` drops below [crate::TRIG_LOW_THRES].
///
/// See also [EnvState] about the first argument `$state`.
/// `$stage_idx` is `$stage_idx + 1` after this stage is finished.
#[macro_export]
macro_rules! env_sustain_stage {
    ($state: expr, $stage_idx: expr, $sustain_value: expr, $gate: expr, $else: block) => {
        if $state.stage == $stage_idx {
            if $gate < $crate::TRIG_LOW_THRES {
                $state.stage += 1;
            }

            $state.current = $sustain_value;
        } else $else
    };
}

/// A retriggerable AD (Attack & Decay) envelope with modifyable shapes for the attack and decay.
///
/// For a more elaborate example see [EnvRetrigAD::tick].
///
///```
/// use synfx_dsp::EnvRetrigAD;
///
/// let mut env = EnvRetrigAD::new();
/// // ..
/// env.set_sample_rate(44100.0);
/// // ..
/// let attack_ms = 3.0;
/// let decay_ms  = 10.0;
/// let attack_shape = 0.5; // 0.5 == linear
/// let decay_shape = 0.5; // 0.5 == linear
/// let trigger_signal = 0.0; // Raise to 1.0 for trigger.
///
/// let (value, retrig) = env.tick(trigger_signal, attack_ms, attack_shape, decay_ms, decay_shape);
/// // ..
///```
///
/// Note: The code for this envelope is used and tested by the `Ad` node of HexoDSP.
#[derive(Debug, Clone)]
pub struct EnvRetrigAD {
    state: EnvState,
    trig: Trigger,
    trig_sig: TrigSignal,
}

impl EnvRetrigAD {
    /// Creates a new instance of the envelope.
    pub fn new() -> Self {
        Self { state: EnvState::new(), trig: Trigger::new(), trig_sig: TrigSignal::new() }
    }

    /// Set the sample rate of the envelope. Unit in samples per second.
    pub fn set_sample_rate(&mut self, srate: f32) {
        self.state.set_sample_rate(srate);
        self.trig_sig.set_sample_rate(srate);
    }

    /// Reset the internal state of the envelope.
    pub fn reset(&mut self) {
        self.state.reset();
        self.trig_sig.reset();
        self.trig.reset();
    }

    /// Computes the next tick for this envelope.
    /// The inputs can be changed on each tick.
    ///
    /// * `trigger` - Trigger input signal, will trigger like [crate::Trigger].
    /// * `attack_ms` - The milliseconds for the attack stage.
    /// * `attack_shape` - The shape for the attack stage.
    ///   Value in the range [[0.0, 1.0]]. 0.5 is linear. See also [crate::sqrt4_to_pow4].
    /// * `decay_ms` - The milliseconds for the decay stage.
    /// * `decay_shape` - The shape for the decay stage.
    ///   Value in the range [[0.0, 1.0]]. 0.5 is linear. See also [crate::sqrt4_to_pow4].
    ///
    /// Returned are two values:
    /// * First the envelope value
    /// * Second a trigger signal at the end of the envelope.
    ///
    ///```
    /// use synfx_dsp::EnvRetrigAD;
    /// let mut env = EnvRetrigAD::new();
    /// env.set_sample_rate(10.0); // Yes, 10 samples per second for testing here :-)
    ///
    /// for _ in 0..2 {
    ///     env.tick(1.0, 500.0, 0.5, 500.0, 0.5);
    /// }
    ///
    /// let (value, _retrig) = env.tick(1.0, 500.0, 0.5, 500.0, 0.5);
    /// assert!((value - 0.6).abs() < 0.0001);
    ///
    /// for _ in 0..5 {
    ///     env.tick(1.0, 500.0, 0.5, 500.0, 0.5);
    /// }
    ///
    /// let (value, _retrig) = env.tick(1.0, 500.0, 0.5, 500.0, 0.5);
    /// assert!((value - 0.2).abs() < 0.0001);
    ///```
    #[inline]
    pub fn tick(
        &mut self,
        trigger: f32,
        attack_ms: f32,
        attack_shape: f32,
        decay_ms: f32,
        decay_shape: f32,
    ) -> (f32, f32) {
        if self.trig.check_trigger(trigger) {
            self.state.trigger();
        }

        if self.state.is_running() {
            env_target_stage_lin_time_adj!(
                self.state,
                0,
                attack_ms,
                0.0,
                1.0,
                |x: f32| sqrt4_to_pow4(x.clamp(0.0, 1.0), attack_shape),
                {
                    env_target_stage!(
                        self.state,
                        2,
                        decay_ms,
                        0.0,
                        |x: f32| sqrt4_to_pow4(x.clamp(0.0, 1.0), decay_shape),
                        {
                            self.trig_sig.trigger();
                            self.state.stop_immediately();
                        }
                    );
                }
            );
        }

        (self.state.current, self.trig_sig.next())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::assert_decimated_slope_feq;
    use crate::assert_vec_feq;

    #[test]
    fn check_hold_stage() {
        let mut state = EnvState::new();
        state.trigger();

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
        state.trigger();

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
        state.trigger();

        env_target_stage!(state, 0, 0.01, 0.6, |x| x, {});
        assert!(state.stage == 2);
        assert!(state.current == 0.6);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
    }

    #[test]
    fn check_short_target_stage() {
        let mut state = EnvState::new();
        state.trigger();

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
    fn check_target_adj_stage() {
        let mut state = EnvState::new();
        state.trigger();

        state.current = 0.0;

        for _ in 0..88 {
            env_target_stage_lin_time_adj!(state, 0, 2.0, 0.0, 0.6, |x| x, {});
            println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
            assert!(state.stage == 1);
        }

        env_target_stage_lin_time_adj!(state, 0, 2.0, 0.0, 0.6, |x| x, {});
        assert!(state.stage == 2);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.current >= 0.5999);
    }

    #[test]
    fn check_target_adj_stage_shortened() {
        let mut state = EnvState::new();
        state.trigger();

        state.current = 0.3;

        for _ in 0..44 {
            env_target_stage_lin_time_adj!(state, 0, 2.0, 0.0, 0.6, |x| x, {});
            println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
            assert!(state.stage == 1);
        }

        env_target_stage_lin_time_adj!(state, 0, 2.0, 0.0, 0.6, |x| x, {});
        assert!(state.stage == 2);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.current >= 0.5999);
    }

    #[test]
    fn check_target_adj_stage_none() {
        let mut state = EnvState::new();
        state.trigger();

        state.current = 0.6;

        env_target_stage_lin_time_adj!(state, 0, 2.0, 0.0, 0.6, |x| x, {});
        assert!(state.stage == 2);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.current >= 0.5999);
    }

    #[test]
    fn check_sustain_stage() {
        let mut state = EnvState::new();
        state.trigger();

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
        state.trigger();

        env_sustain_stage!(state, 0, 0.5, 0.0, {});
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 1);
        assert!((state.current - 0.5).abs() < 0.0001);
    }

    #[test]
    fn check_ahd_env() {
        let mut state = EnvState::new();
        state.set_sample_rate(48000.0);
        state.trigger();

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

    #[test]
    fn check_env_ad() {
        let mut env = EnvRetrigAD::new();

        env.set_sample_rate(10.0);

        let mut values = vec![];
        let mut retrig_index = -1;
        for i in 0..16 {
            let (value, retrig) = env.tick(1.0, 1000.0, 0.5, 500.0, 0.5);
            values.push(value);
            if retrig > 0.0 {
                retrig_index = i as i32;
            }
        }

        assert_vec_feq!(
            values,
            vec![
                0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.70000005, 0.8000001, 0.9000001, 1.0, 0.8, 0.6,
                0.39999998, 0.19999999, 0.0, 0.0
            ]
        );

        assert_eq!(retrig_index, 15);
    }

    #[test]
    fn check_env_ad_shaped() {
        let mut env = EnvRetrigAD::new();

        env.set_sample_rate(10.0);

        let mut values = vec![];
        let mut retrig_index = -1;
        for i in 0..16 {
            let (value, retrig) = env.tick(1.0, 1000.0, 0.7, 500.0, 0.3);
            values.push(value);
            if retrig > 0.0 {
                retrig_index = i as i32;
            }
        }

        assert_vec_feq!(
            values,
            vec![
                0.2729822, 0.39777088, 0.49817806, 0.58596444, 0.6656854, 0.7396773, 0.809328,
                0.8755418, 0.93894666, 1.0, 0.928, 0.79199994, 0.592, 0.32799995, 0.0, 0.0
            ]
        );

        assert_eq!(retrig_index, 15);
    }
}
