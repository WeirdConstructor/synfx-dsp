// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

use crate::TRIG_LOW_THRES;

struct EnvState {
    srate_ms: f32,
    stage: u32,
    phase: f32,
    inc: f32,
    start: f32,
    current: f32,
}

impl EnvState {
    pub fn new() -> Self {
        Self {
            srate_ms: 44100.0 / 1000.0,
            stage: 0,
            phase: 0.0,
            inc: 0.0,
            start: 0.0,
            current: 0.0,
        }
    }
}

#[macro_export]
macro_rules! env_hold_stage {
    ($state: expr, $stage_idx: expr, $time_ms: expr) => {
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
        }
    };
}

#[macro_export]
macro_rules! env_target_stage {
    ($state: expr, $stage_idx: expr, $time_ms: expr, $value: expr, $shape_fn: expr) => {
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
                $state.current = ($shape_fn)($state.start * (1.0 - $state.phase) + $state.phase * $value);
            }
        }
    };
}

#[macro_export]
macro_rules! env_sustain_stage {
    ($state: expr, $stage_idx: expr, $sustain_value: expr, $gate: expr) => {
        if $state.stage == $stage_idx {
            if $gate < TRIG_LOW_THRES {
                $state.stage += 1;
            }

            $state.current = $sustain_value;
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_hold_stage() {
        let mut state = EnvState::new();

        state.current = 0.6;
        for _ in 0..88 {
            env_hold_stage!(state, 0, 2.0);
            println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
            assert!(state.stage == 1);
            assert!(state.current > 0.5);
        }

        env_hold_stage!(state, 0, 2.0);
        assert!(state.stage == 2);
        assert!(state.current > 0.5);
    }

    #[test]
    fn check_target_stage() {
        let mut state = EnvState::new();

        for _ in 0..88 {
            env_target_stage!(state, 0, 2.0, 0.6, |x| x);
            assert!(state.stage == 1);
            println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        }

        env_target_stage!(state, 0, 2.0, 0.6, |x| x);
        assert!(state.stage == 2);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.current >= 0.5999);
    }

    #[test]
    fn check_very_short_target_stage() {
        let mut state = EnvState::new();

        env_target_stage!(state, 0, 0.01, 0.6, |x| x);
        assert!(state.stage == 2);
        assert!(state.current == 0.6);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
    }

    #[test]
    fn check_short_target_stage() {
        let mut state = EnvState::new();

        env_target_stage!(state, 0, 0.03, 0.6, |x| x);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 1);
        assert!((state.current - 0.4535).abs() < 0.0001);

        env_target_stage!(state, 0, 0.03, 0.6, |x| x);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 2);
        assert!(state.current == 0.6);
    }

    #[test]
    fn check_sustain_stage() {
        let mut state = EnvState::new();

        env_sustain_stage!(state, 0, 0.5, 1.0);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 0);
        assert!((state.current - 0.5).abs() < 0.0001);

        env_sustain_stage!(state, 0, 0.5, 1.0);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 0);
        assert!((state.current - 0.5).abs() < 0.0001);

        env_sustain_stage!(state, 0, 0.5, 0.0);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 1);
        assert!((state.current - 0.5).abs() < 0.0001);
    }

    #[test]
    fn check_sustain_stage_short() {
        let mut state = EnvState::new();

        env_sustain_stage!(state, 0, 0.5, 0.0);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, state.current);
        assert!(state.stage == 1);
        assert!((state.current - 0.5).abs() < 0.0001);
    }
}
