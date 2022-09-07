0.5.5 (unreleased)
==================

* Feature: `EnvState` got more methods to control the envelope (such as triggering/retriggering).
* Feature: `env_target_stage_lin_time_adj` added, for a retriggerable attack stage.

0.5.4 (2022-08-28)
==================

* Bugfix: `env_sustain_stage` referenced `TRIG_LOW_THRES` without any proper crate prefix.

0.5.3 (2022-08-23)
==================

* Feature: Added `GateSignal` for generating gate signals of a specified length in
milliseconds.
* Feature: Added some macros useful for testing DSP code, like `assert_vec_feq`, `assert_decimated_feq`,
`assert_decimated_slope_feq` and some others.
* Feature: Added envelope toolkit in the `synfx_dsp::env` module. You can piece together your
own envelope using a few macros such as `env_target_stage`, `env_hold_stage` and `env_sustain_stage`.

0.5.2 (2022-08-06)
==================

* Feature: Pulled over `AtomicFloat` and `AtomicFloatPair` from HexoDSP.

0.5.1 (2022-08-05)
==================

* Bugfix: Forgot to link in the dattorro.rs with the reverb algorithm.
* Change: Moved test for delay buffer into this crate.

0.5.0 (2022-08-05)
==================

* Initial release.
