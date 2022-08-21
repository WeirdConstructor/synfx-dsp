// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

struct EnvState {
    srate_ms: f32,
    stage: u32,
    phase: f32,
    inc: f32,
    start: f32,
}

impl EnvState {
    pub fn new() -> Self {
        Self {
            srate_ms: 44100.0 / 1000.0,
            stage: 0,
            phase: 0.0,
            inc: 0.0,
            start: 0.0,
        }
    }
}

#[macro_export]
macro_rules! env_delay_stage {
    ($state: expr, $stage_idx: expr, $time_ms: expr, $out: expr) => {
        if $state.stage == $stage_idx {
            $state.phase = 0.0;
            $state.stage += 1;
            $state.start = $out;
        } else if $state.stage == ($stage_idx + 1) {
            let inc = 1.0 / ($time_ms * $state.srate_ms);
            $state.phase += inc;
            if ($state.phase + inc) >= 1.0 {
                $state.stage += 1;
            }
            $out = $state.start;
        }
    };
}

#[macro_export]
macro_rules! env_target_stage {
    ($state: expr, $stage_idx: expr, $time_ms: expr, $value: expr, $out: expr) => {
        if $state.stage == $stage_idx {
            let time_samples = $time_ms * $state.srate_ms;
            // *2.0 to make it stop one step earlier, to really reach the target value:
            $state.phase = 0.0;
            $state.start = $out;
            $state.stage += 1;
        } else if $state.stage == ($stage_idx + 1) {
            let inc = 1.0 / ($time_ms * $state.srate_ms);
            $state.phase += inc;
            if ($state.phase + inc) >= 1.0 {
                $state.stage += 1;
                $out = $value;
            } else {
                $out = $state.start * (1.0 - $state.phase) + $state.phase * $value;
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_delay_stage() {
        let mut state = EnvState::new();

        let mut v = 0.6;
        for _ in 0..88 {
            env_delay_stage!(state, 0, 2.0, v);
            println!("V[{:6.4}]: {:6.4}", state.phase, v);
            assert!(state.stage == 1);
            assert!(v > 0.5);
        }

        env_delay_stage!(state, 0, 2.0, v);
        assert!(state.stage == 2);
        assert!(v > 0.5);
    }

    #[test]
    fn check_target_stage() {
        let mut state = EnvState::new();

        let mut v = 0.0;
        for _ in 0..88 {
            env_target_stage!(state, 0, 2.0, 0.6, v);
            assert!(state.stage == 1);
            println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, v);
        }

        env_target_stage!(state, 0, 2.0, 0.6, v);
        assert!(state.stage == 2);
        println!("V[{:6.4} / {}]: {:6.4}", state.phase, state.stage, v);
        assert!(v >= 0.5999);
    }
}
