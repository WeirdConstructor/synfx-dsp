// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.

/*! Provides you with some useful macros for testing DSP code.

*/

/// This macro allows you to float compare two vectors to a precision of `0.0001`.
#[macro_export]
macro_rules! assert_vec_feq {
    ($vec:expr, $cmp_vec:expr) => {
        let cmp_vec = $cmp_vec;
        let res: Vec<f32> = $vec.iter().copied().collect();

        for (i, (s, scmp)) in res.iter().zip(cmp_vec.iter()).enumerate() {
            if (s - scmp).abs() > 0.0001 {
                panic!(
                    r#"
table_left: {:?}

table_right: {:?}

assertion failed: `(left[{}] == right[{}])`
      left: `{:?}`,
     right: `{:?}`"#,
                    &res[i..],
                    &(cmp_vec[i..]),
                    i,
                    i,
                    s,
                    scmp
                )
            }
        }
    };
}

/// This macro allows you to float compare two vectors to a precision of `0.0001`,
/// only every `$decimate` element will be looked at though. Useful for keeping the fixed
/// value tables in your DSP code small.
#[macro_export]
macro_rules! assert_decimated_feq {
    ($vec:expr, $decimate:expr, $cmp_vec:expr) => {
        let cmp_vec = $cmp_vec;
        let res: Vec<f32> = $vec.iter().step_by($decimate).copied().collect();

        for (i, (s, scmp)) in res.iter().zip(cmp_vec.iter()).enumerate() {
            if (s - scmp).abs() > 0.0001 {
                panic!(
                    r#"
table_left: {:?}

table_right: {:?}

assertion failed: `(left[{}] == right[{}])`
      left: `{:?}`,
     right: `{:?}`"#,
                    &res[i..],
                    &(cmp_vec[i..]),
                    i,
                    i,
                    s,
                    scmp
                )
            }
        }
    };
}

/// Calculates the (linear) slope between consequtive values in `$vec` and compares the slopes
/// with `$cmp_vec` with a precision of `0.0001`.
#[macro_export]
macro_rules! assert_slope_feq {
    ($vec:expr, $cmp_vec:expr) => {
        let cmp_vec = $cmp_vec;
        let mut res: Vec<f32> = vec![];
        let mut prev = 0.0;
        for (i, s) in $vec.iter().enumerate() {
            let delta = *s - prev;
            if i > 0 {
                res.push(delta);
            }
            prev = *s;
        }

        let res: Vec<f32> = res.iter().copied().collect();

        for (i, (s, scmp)) in res.iter().zip(cmp_vec.iter()).enumerate() {
            if (s - scmp).abs() > 0.0001 {
                panic!(
                    r#"
table_left: {:?}

table_right: {:?}

assertion failed: `(left[{}] == right[{}])`
      left: `{:?}`,
     right: `{:?}`"#,
                    &res[i..],
                    &(cmp_vec[i..]),
                    i,
                    i,
                    s,
                    scmp
                )
            }
        }
    };
}

/// Calculates the (linear) slope between consequtive values in `$vec` and compares the slopes
/// with `$cmp_vec` with a precision of `0.0001`. This macro only looks at every `$decimate` slope.
#[macro_export]
macro_rules! assert_decimated_slope_feq {
    ($vec:expr, $decimate:expr, $cmp_vec:expr) => {
        let cmp_vec = $cmp_vec;
        let mut res: Vec<f32> = vec![];
        let mut prev = 0.0;
        for (i, s) in $vec.iter().enumerate() {
            let delta = *s - prev;
            if i > 0 {
                res.push(delta);
            }
            prev = *s;
        }

        let res: Vec<f32> = res.iter().step_by($decimate).copied().collect();

        for (i, (s, scmp)) in res.iter().zip(cmp_vec.iter()).enumerate() {
            if (s - scmp).abs() > 0.0001 {
                panic!(
                    r#"
table_left: {:?}

table_right: {:?}

assertion failed: `(left[{}] == right[{}])`
      left: `{:?}`,
     right: `{:?}`"#,
                    &res[i..],
                    &(cmp_vec[i..]),
                    i,
                    i,
                    s,
                    scmp
                )
            }
        }
    };
}

/// Calculates the (linear) slope between consequtive values in `$vec` and compares the slopes
/// with `$cmp_vec` with a precision of `0.0000001`. This macro only looks at every `$decimate` slope.
#[macro_export]
macro_rules! assert_decimated_slope_feq_fine {
    ($vec:expr, $decimate:expr, $cmp_vec:expr) => {
        let cmp_vec = $cmp_vec;
        let mut res: Vec<f32> = vec![];
        let mut prev = 0.0;
        for (i, s) in $vec.iter().enumerate() {
            let delta = *s - prev;
            if i > 0 {
                res.push(delta);
            }
            prev = *s;
        }

        let res: Vec<f32> = res.iter().step_by($decimate).copied().collect();

        for (i, (s, scmp)) in res.iter().zip(cmp_vec.iter()).enumerate() {
            if (s - scmp).abs() > 0.0000001 {
                panic!(
                    r#"
table_left: {:?}

table_right: {:?}

assertion failed: `(left[{}] == right[{}])`
      left: `{:?}`,
     right: `{:?}`"#,
                    &res[i..],
                    &(cmp_vec[i..]),
                    i,
                    i,
                    s,
                    scmp
                )
            }
        }
    };
}

/// Calculates the (linear) slope between consequtive values in `$vec` and compares the slopes
/// with `$cmp_vec` with a precision of `0.000000001`. This macro only looks at every `$decimate` slope.
#[macro_export]
macro_rules! assert_decimated_slope_feq_sfine {
    ($vec:expr, $decimate:expr, $cmp_vec:expr) => {
        let cmp_vec = $cmp_vec;
        let mut res: Vec<f32> = vec![];
        let mut prev = 0.0;
        for (i, s) in $vec.iter().enumerate() {
            let delta = *s - prev;
            if i > 0 {
                res.push(delta);
            }
            prev = *s;
        }

        let res: Vec<f32> = res.iter().step_by($decimate).copied().collect();

        for (i, (s, scmp)) in res.iter().zip(cmp_vec.iter()).enumerate() {
            if (s - scmp).abs() > 0.000000001 {
                panic!(
                    r#"
table_left: {:?}

table_right: {:?}

assertion failed: `(left[{}] == right[{}])`
      left: `{:?}`,
     right: `{:?}`"#,
                    &res[i..],
                    &(cmp_vec[i..]),
                    i,
                    i,
                    s,
                    scmp
                )
            }
        }
    };
}
