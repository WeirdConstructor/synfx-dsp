// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of HexoDSP. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

///! Contains various utilities for trigger signals in a modular synthesizer.
///
/// There are also clock synchronizing helpers in here like [TriggerPhaseClock]
/// or [TriggerSampleClock].

/// A-100 Eurorack states, that a trigger is usually 2-10 milliseconds.
pub const TRIG_SIGNAL_LENGTH_MS: f32 = 2.0;

/// The lower threshold for the schmidt trigger to reset.
pub const TRIG_LOW_THRES: f32 = 0.25;
/// The threshold, once reached, will cause a trigger event and signals
/// a logical '1'. Anything below this is a logical '0'.
pub const TRIG_HIGH_THRES: f32 = 0.5;

/// Trigger signal generator for HexoDSP nodes.
///
/// A trigger in HexoSynth and HexoDSP is commonly 2.0 milliseconds.
/// This generator generates a trigger signal when [TrigSignal::trigger] is called.
#[derive(Debug, Clone, Copy)]
pub struct TrigSignal {
    length: u32,
    scount: u32,
}

impl TrigSignal {
    /// Create a new trigger generator
    pub fn new() -> Self {
        Self { length: ((44100.0 * TRIG_SIGNAL_LENGTH_MS) / 1000.0).ceil() as u32, scount: 0 }
    }

    /// Reset the trigger generator.
    pub fn reset(&mut self) {
        self.scount = 0;
    }

    /// Set the sample rate to calculate the amount of samples for the trigger signal.
    pub fn set_sample_rate(&mut self, srate: f32) {
        self.length = ((srate * TRIG_SIGNAL_LENGTH_MS) / 1000.0).ceil() as u32;
        self.scount = 0;
    }

    /// Enable sending a trigger impulse the next time [TrigSignal::next] is called.
    #[inline]
    pub fn trigger(&mut self) {
        self.scount = self.length;
    }

    /// Trigger signal output.
    #[inline]
    pub fn next(&mut self) -> f32 {
        if self.scount > 0 {
            self.scount -= 1;
            1.0
        } else {
            0.0
        }
    }
}

impl Default for TrigSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Signal change detector that emits a trigger when the input signal changed.
///
/// This is commonly used for control signals. It has not much use for audio signals.
#[derive(Debug, Clone, Copy)]
pub struct ChangeTrig {
    ts: TrigSignal,
    last: f32,
}

impl ChangeTrig {
    /// Create a new change detector
    pub fn new() -> Self {
        Self {
            ts: TrigSignal::new(),
            last: -100.0, // some random value :-)
        }
    }

    /// Reset internal state.
    pub fn reset(&mut self) {
        self.ts.reset();
        self.last = -100.0;
    }

    /// Set the sample rate for the trigger signal generator
    pub fn set_sample_rate(&mut self, srate: f32) {
        self.ts.set_sample_rate(srate);
    }

    /// Feed a new input signal sample.
    ///
    /// The return value is the trigger signal.
    #[inline]
    pub fn next(&mut self, inp: f32) -> f32 {
        if (inp - self.last).abs() > std::f32::EPSILON {
            self.ts.trigger();
            self.last = inp;
        }

        self.ts.next()
    }
}

impl Default for ChangeTrig {
    fn default() -> Self {
        Self::new()
    }
}

/// Trigger signal detector for HexoDSP.
///
/// Whenever you need to detect a trigger on an input you can use this component.
/// A trigger in HexoDSP is any signal over [TRIG_HIGH_THRES]. The internal state is
/// resetted when the signal drops below [TRIG_LOW_THRES].
#[derive(Debug, Clone, Copy)]
pub struct Trigger {
    triggered: bool,
}

impl Trigger {
    /// Create a new trigger detector.
    pub fn new() -> Self {
        Self { triggered: false }
    }

    /// Reset the internal state of the trigger detector.
    #[inline]
    pub fn reset(&mut self) {
        self.triggered = false;
    }

    /// Checks the input signal for a trigger and returns true when the signal
    /// surpassed [TRIG_HIGH_THRES] and has not fallen below [TRIG_LOW_THRES] yet.
    #[inline]
    pub fn check_trigger(&mut self, input: f32) -> bool {
        if self.triggered {
            if input <= TRIG_LOW_THRES {
                self.triggered = false;
            }

            false
        } else if input > TRIG_HIGH_THRES {
            self.triggered = true;
            true
        } else {
            false
        }
    }
}

/// Trigger signal detector with custom range.
///
/// Whenever you need to detect a trigger with a custom threshold.
#[derive(Debug, Clone, Copy)]
pub struct CustomTrigger {
    triggered: bool,
    low_thres: f32,
    high_thres: f32,
}

impl CustomTrigger {
    /// Create a new trigger detector.
    pub fn new(low_thres: f32, high_thres: f32) -> Self {
        Self { triggered: false, low_thres, high_thres }
    }

    pub fn set_threshold(&mut self, low_thres: f32, high_thres: f32) {
        self.low_thres = low_thres;
        self.high_thres = high_thres;
    }

    /// Reset the internal state of the trigger detector.
    #[inline]
    pub fn reset(&mut self) {
        self.triggered = false;
    }

    /// Checks the input signal for a trigger and returns true when the signal
    /// surpassed the high threshold and has not fallen below low threshold yet.
    #[inline]
    pub fn check_trigger(&mut self, input: f32) -> bool {
        //        println!("TRIG CHECK: {} <> {}", input, self.high_thres);
        if self.triggered {
            if input <= self.low_thres {
                self.triggered = false;
            }

            false
        } else if input > self.high_thres {
            self.triggered = true;
            true
        } else {
            false
        }
    }
}

/// Generates a phase signal from a trigger/gate input signal.
///
/// This helper allows you to measure the distance between trigger or gate pulses
/// and generates a phase signal for you that increases from 0.0 to 1.0.
#[derive(Debug, Clone, Copy)]
pub struct TriggerPhaseClock {
    clock_phase: f64,
    clock_inc: f64,
    prev_trigger: bool,
    clock_samples: u32,
}

impl TriggerPhaseClock {
    /// Create a new phase clock.
    pub fn new() -> Self {
        Self { clock_phase: 0.0, clock_inc: 0.0, prev_trigger: true, clock_samples: 0 }
    }

    /// Reset the phase clock.
    #[inline]
    pub fn reset(&mut self) {
        self.clock_samples = 0;
        self.clock_inc = 0.0;
        self.prev_trigger = true;
        self.clock_samples = 0;
    }

    /// Restart the phase clock. It will count up from 0.0 again on [TriggerPhaseClock::next_phase].
    #[inline]
    pub fn sync(&mut self) {
        self.clock_phase = 0.0;
    }

    /// Generate the phase signal of this clock.
    ///
    /// * `clock_limit` - The maximum number of samples to detect two trigger signals in.
    /// * `trigger_in` - Trigger signal input.
    #[inline]
    pub fn next_phase(&mut self, clock_limit: f64, trigger_in: f32) -> f64 {
        if self.prev_trigger {
            if trigger_in <= TRIG_LOW_THRES {
                self.prev_trigger = false;
            }
        } else if trigger_in > TRIG_HIGH_THRES {
            self.prev_trigger = true;

            if self.clock_samples > 0 {
                self.clock_inc = 1.0 / (self.clock_samples as f64);
            }

            self.clock_samples = 0;
        }

        self.clock_samples += 1;

        self.clock_phase += self.clock_inc;
        self.clock_phase = self.clock_phase % clock_limit;

        self.clock_phase
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TriggerSampleClock {
    prev_trigger: bool,
    clock_samples: u32,
    counter: u32,
}

impl TriggerSampleClock {
    pub fn new() -> Self {
        Self { prev_trigger: true, clock_samples: 0, counter: 0 }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.clock_samples = 0;
        self.counter = 0;
    }

    #[inline]
    pub fn next(&mut self, trigger_in: f32) -> u32 {
        if self.prev_trigger {
            if trigger_in <= TRIG_LOW_THRES {
                self.prev_trigger = false;
            }
        } else if trigger_in > TRIG_HIGH_THRES {
            self.prev_trigger = true;
            self.clock_samples = self.counter;
            self.counter = 0;
        }

        self.counter += 1;

        self.clock_samples
    }
}

