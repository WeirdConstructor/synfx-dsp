0.5.6 (unreleased)
==================

* Feature: Added the `fh_va` (`LadderFilter`, `Svf`, `SallenKey`) virtual analog
filter code by Fredemus.

0.5.5 (2022-11-01)
==================

* Change: Requires nightly Rust due to SIMD features.
* Feature: `EnvState` got more methods to control the envelope (such as triggering/retriggering).
* Feature: `env_target_stage_lin_time_adj` added, for a retriggerable envelope.
* Feature: Added an attack decay envelope implementation with `EnvRetrigAD`.
* Feature: Added `coef2gain_db` and `gain_db2coeff` for converting decibel gain values to coefficients.
* Documentation: I've added every piece of foreign code and it's license to the README and lib.rs
documentation.

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
