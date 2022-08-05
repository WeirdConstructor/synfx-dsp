// Copyright (c) 2021-2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

//! Random number generators and utilities.
/// Be aware that some might need some initialization function!

use std::cell::RefCell;

/// A wavetable filled entirely with white noise.
/// Don't forget to call [init_white_noise_tab] before using it.
static mut WHITE_NOISE_TAB: [f64; 1024] = [0.0; 1024];

#[allow(rustdoc::private_intra_doc_links)]
/// Initializes [WHITE_NOISE_TAB]
pub fn init_white_noise_tab() {
    let mut rng = RandGen::new();
    unsafe {
        for i in 0..WHITE_NOISE_TAB.len() {
            WHITE_NOISE_TAB[i as usize] = rng.next_open01();
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
/// Random number generator based on xoroshiro128.
/// Requires two internal state variables.
/// You may prefer [SplitMix64] or [Rng] which only use one `u64` as state.
pub struct RandGen {
    r: [u64; 2],
}

// Taken from xoroshiro128 crate under MIT License
// Implemented by Matthew Scharley (Copyright 2016)
// https://github.com/mscharley/rust-xoroshiro128
/// Given the mutable `state` generates the next pseudo random number.
pub fn next_xoroshiro128(state: &mut [u64; 2]) -> u64 {
    let s0: u64 = state[0];
    let mut s1: u64 = state[1];
    let result: u64 = s0.wrapping_add(s1);

    s1 ^= s0;
    state[0] = s0.rotate_left(55) ^ s1 ^ (s1 << 14); // a, b
    state[1] = s1.rotate_left(36); // c

    result
}

// Taken from rand::distributions
// Licensed under the Apache License, Version 2.0
// Copyright 2018 Developers of the Rand project.
/// Maps any `u64` to a `f64` in the open interval `[0.0, 1.0)`.
pub fn u64_to_open01(u: u64) -> f64 {
    use core::f64::EPSILON;
    let float_size = std::mem::size_of::<f64>() as u32 * 8;
    let fraction = u >> (float_size - 52);
    let exponent_bits: u64 = (1023 as u64) << 52;
    f64::from_bits(fraction | exponent_bits) - (1.0 - EPSILON / 2.0)
}

impl RandGen {
    pub fn new() -> Self {
        RandGen { r: [0x193a6754a8a7d469, 0x97830e05113ba7bb] }
    }

    /// Next random unsigned 64bit integer.
    pub fn next(&mut self) -> u64 {
        next_xoroshiro128(&mut self.r)
    }

    /// Next random float between `[0.0, 1.0)`.
    pub fn next_open01(&mut self) -> f64 {
        u64_to_open01(self.next())
    }
}

#[derive(Debug, Copy, Clone)]
/// Random number generator based on [SplitMix64].
/// Requires two internal state variables. You may prefer [SplitMix64] or [Rng].
pub struct Rng {
    sm: SplitMix64,
}

impl Rng {
    pub fn new() -> Self {
        Self { sm: SplitMix64::new(0x193a67f4a8a6d769) }
    }

    pub fn seed(&mut self, seed: u64) {
        self.sm = SplitMix64::new(seed);
    }

    #[inline]
    pub fn next(&mut self) -> f32 {
        self.sm.next_open01() as f32
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.sm.next_u64()
    }
}

thread_local! {
    static GLOBAL_RNG: RefCell<Rng> = RefCell::new(Rng::new());
}

#[inline]
pub fn rand_01() -> f32 {
    GLOBAL_RNG.with(|r| r.borrow_mut().next())
}

#[inline]
pub fn rand_u64() -> u64 {
    GLOBAL_RNG.with(|r| r.borrow_mut().next_u64())
}

// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//- splitmix64 (http://xoroshiro.di.unimi.it/splitmix64.c)
//
/// A splitmix64 random number generator.
///
/// The splitmix algorithm is not suitable for cryptographic purposes, but is
/// very fast and has a 64 bit state.
///
/// The algorithm used here is translated from [the `splitmix64.c`
/// reference source code](http://xoshiro.di.unimi.it/splitmix64.c) by
/// Sebastiano Vigna. For `next_u32`, a more efficient mixing function taken
/// from [`dsiutils`](http://dsiutils.di.unimi.it/) is used.
#[derive(Debug, Copy, Clone)]
pub struct SplitMix64(pub u64);

/// Internal random constant for [SplitMix64].
const PHI: u64 = 0x9e3779b97f4a7c15;

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn new_from_i64(seed: i64) -> Self {
        Self::new(u64::from_be_bytes(seed.to_be_bytes()))
    }

    pub fn new_time_seed() -> Self {
        use std::time::SystemTime;

        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => Self::new(n.as_secs() as u64),
            Err(_) => Self::new(123456789),
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(PHI);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    #[inline]
    pub fn next_i64(&mut self) -> i64 {
        i64::from_be_bytes(self.next_u64().to_be_bytes())
    }

    #[inline]
    pub fn next_open01(&mut self) -> f64 {
        u64_to_open01(self.next_u64())
    }
}

