// Copyright (c) 2022 Weird Constructor <weirdconstructor@gmail.com>
// This file is a part of synfx-dsp. Released under GPL-3.0-or-later.
// See README.md and COPYING for details.
//
// This file contains the VA filter code of Fredemus' aka Frederik Halkjær aka RocketPhysician
// VA filter implementation.
// Copied under GPL-3.0-or-later from https://github.com/Fredemus/va-filter

use crate::fh_va::{DKSolver, FilterParams, SvfMode};
use std::sync::Arc;
use std::simd::f32x4;

/// This is a 2-pole multimode filter.
///
/// This is a 2-pole multimode filter loosely based on the one found in the edp
/// wasp synthesizer. It's a good all-around filter that distorts nicely and
/// keeps resonance well at high levels.
///
/// It's capable of outputting all basic filter modes (lowpass, highpass,
/// bandpass, notch, etc.) and self-oscillation.
///
/// OTA core, nonlinear op-amp buffers. The EDP wasp uses inverters as a weird
/// extremely nonlinear op-amp buffer, but I haven't looked into how to model
/// that (in a way that converges well) yet.  Resonance is limited by a diode
/// clipper on the damping feedback, boosting it when gain is high, since it'd
/// otherwise disappear because of the opamp nonlinearities, which would lead
/// to the resonance completely dominating the signal.
///
/// Its convergence is generally good.  The convergence gets a lot better when
/// oversampled 2x or more, which I recommend anyway since it distorts.
///
/// Circuit solved by Holters & Zölzer's generalization of the DK-method. This
/// method has a lot of advantages compared to the other approach, namely it's
/// much better equipped for handling nonlinear voltage-controlled voltage
/// sources such as op-amps and jacobian matrices are only necessary on a
/// per-component basis, meaning it's not necessary to solve the whole system
/// each iteration, speeding up iterations significantly.  Special thanks to
/// Martin Holters and his amazing circuit emulation tool
/// [ACME](https://github.com/HSU-ANT/ACME.jl) for the great work on circuit
/// emulation and answering my questions when I got stuck.
///
/// The fast version is optimized by removing unnecessary operations and
/// replacing the general solver with an analytic solution of the specific
/// model.  At some point I'll look into how a simd-optimized version would
/// compare, since most of the operations are dot products anyway, but the
/// current fast version is definitely fast enough for real-time use in DAW
/// projects.  Sadly convergence varies too much for using simd-lanes for
/// processing left and right at the same time to bring a big performance
/// benefit.
#[derive(Debug, Clone)]
pub struct Svf {
    filters: [SvfCoreFast; 2],
}

const N_P: usize = 3;
const N_N: usize = 4;
const P_LEN: usize = 8;
const N_OUTS: usize = 3;
const N_STATES: usize = 2;
const TOL: f64 = 1e-5;

impl Svf {
    pub fn new(params: Arc<FilterParams>) -> Self {
        Self { filters: [SvfCoreFast::new(params.clone()), SvfCoreFast::new(params)] }
    }
    /// Process a stereo sample.
    pub fn process(&mut self, input: f32x4) -> f32x4 {
        f32x4::from_array([
            self.filters[0].tick_dk(input[0]),
            self.filters[1].tick_dk(input[1]),
            0.,
            0.,
        ])
    }
    /// Call this whenver the resonance or cutoff frequency of the [FilterParams] change.
    pub fn update(&mut self) {
        self.filters[0].update_matrices();
        self.filters[1].update_matrices();
    }
    /// Reset the filter.
    pub fn reset(&mut self) {
        self.filters[0].reset();
        self.filters[1].reset();
    }
}

#[derive(Debug, Clone)]
pub struct SvfCoreFast {
    pub params: Arc<FilterParams>,
    pub vout: [f32; N_OUTS],
    pub s: [f32; N_STATES],

    // the not-trivial coefficients in the model
    c1: f64,
    c2: f64,
    // for storing the jacobian for the q (p + dot(z, fq) vector
    jq: [f64; P_LEN],
    solver: DKSolver<N_N, N_P, P_LEN>,
}

impl SvfCoreFast {
    pub fn new(params: Arc<FilterParams>) -> Self {
        let fs = params.sample_rate;
        let g = (std::f32::consts::PI * 1000. / (fs as f32)).tan();
        let res = 0.1;
        let g_f64 = g as f64;
        let res_f64 = res as f64;

        let mut a = Self {
            params,
            vout: [0.; N_OUTS],
            s: [0.; 2],

            c1: 2. * g_f64,
            c2: res_f64,

            jq: [0., -1., 0., -1., 0., -1., 0., -1.],
            solver: DKSolver::new(),
        };
        a.reset();
        a
    }

    pub fn update_matrices(&mut self) {
        let g = self.params.g * 2.;
        let res = self.params.zeta;
        let g_f64 = g as f64;
        let res_f64 = res as f64;

        self.c1 = 2. * g_f64;
        self.c2 = res_f64;
    }
    pub fn tick_dk(&mut self, input: f32) -> f32 {
        // -input since the svf inverts it
        let input = -input * (self.params.drive);

        let mut p = [0.; N_P];

        p[0] = -self.s[0] as f64;
        p[1] = -self.s[1] as f64;
        p[2] = input as f64;

        // find nonlinear contributions (solver.z), applying homotopy if it fails to converge
        self.homotopy_solver(p);
        // self.nonlinear_contribs(p);

        self.vout[0] = self.solver.z[3] as f32;
        self.vout[1] = self.solver.z[2] as f32;
        self.vout[2] = self.solver.z[1] as f32;

        self.s[0] = self.s[0] - 2. * (self.c1 * self.solver.z[1]) as f32;
        self.s[1] = self.s[1] - 2. * (self.c1 * self.solver.z[2]) as f32;

        self.get_output(input, self.params.zeta)
    }

    pub fn homotopy_solver(&mut self, p: [f64; N_P]) {
        self.nonlinear_contribs(p);
        // if the newton solver failed to converge, apply homotopy
        if self.solver.resmaxabs >= TOL {
            // println!("needs homotopy");
            let mut a = 0.5;
            let mut best_a = 0.;
            while best_a < 1. {
                let mut pa = self.solver.last_p;

                for i in 0..pa.len() {
                    pa[i] = pa[i] * (1. - a);
                    pa[i] = pa[i] + a * p[i];
                }
                self.nonlinear_contribs(pa);
                if self.solver.resmaxabs < TOL {
                    best_a = a;
                    a = 1.0;
                } else {
                    let new_a = (a + best_a) / 2.;
                    if !(best_a < new_a && new_a < a) {
                        // no values between a and best_a. This means the homotopy failed to find an in-between value for the solution
                        break;
                    }
                    a = new_a;
                }
            }
        }
    }

    // uses newton's method to find the nonlinear contributions in the circuit. Not guaranteed to converge
    fn nonlinear_contribs(&mut self, p: [f64; N_P]) {
        self.solver.p_full[2] = p[0];
        self.solver.p_full[4] = p[1];
        self.solver.p_full[7] = p[2];

        let mut tmp_np = [0.; N_P];

        tmp_np[0] = p[0] - self.solver.last_p[0];
        tmp_np[1] = p[1] - self.solver.last_p[1];
        tmp_np[2] = p[2] - self.solver.last_p[2];

        let mut tmp_nn = [
            0.,
            self.jq[2] * tmp_np[0],
            self.jq[4] * tmp_np[1],
            -tmp_np[2],
        ];
        tmp_nn = self.solve_lin_equations(tmp_nn);
        for i in 0..N_N {
            self.solver.z[i] = self.solver.last_z[i] - tmp_nn[i];
        }

        for _plsconverge in 0..100 {
            self.evaluate_nonlinearities(self.solver.z);

            self.solver.resmaxabs = 0.;
            for x in &self.solver.residue {
                if x.is_finite() {
                    if x.abs() > self.solver.resmaxabs {
                        self.solver.resmaxabs = x.abs();
                    }
                } else {
                    // if any of the residue have become NaN/inf, stop early.
                    // If using the homotopy solver, it will kick in and find an alternate, slower path to convergence
                    self.solver.resmaxabs = 1000.;
                    return;
                }
            }

            // self.solver.set_lin_solver(self.solver.j);
            if self.solver.resmaxabs < TOL {
                // dbg!(_plsconverge);
                break;
            }

            // update z with the linsolver according to the residue
            tmp_nn = self.solve_lin_equations(self.solver.residue);
            // tmp_nn = self.solver.solve_linear_equations(self.solver.residue);

            for i in 0..self.solver.z.len() {
                self.solver.z[i] -= tmp_nn[i];
            }
        }
        if self.solver.resmaxabs < TOL {
            self.solver.set_extrapolation_origin(p, self.solver.z);
        }
        // else {
        // panic!("failed to converge. residue: {:?}", self.solver.residue);
        // println!("failed to converge. residue: {:?}", self.solver.residue);
        // }
    }
    #[inline]
    fn evaluate_nonlinearities(&mut self, z: [f64; N_N]) {
        let mut q = self.solver.p_full;

        q[0] += z[0];
        q[1] += z[1];
        q[2] += self.c1 * z[1] - z[2];
        q[3] += z[2];
        q[4] += self.c1 * z[2] - z[3];
        q[5] += z[3];
        q[6] += -z[0] - z[2];
        q[7] += 4. * z[0] + z[1] + self.c2 * z[2] + 2. * z[3];
        // q[7] += 3. * z[0] + z[1] + self.c2 * z[2] + z[3];

        let (res1, jq1) = self.solver.eval_opamp(q[0], q[1]);
        let (res2, jq2) = self.solver.eval_opamp(q[2], q[3]);
        let (res3, jq3) = self.solver.eval_opamp(q[4], q[5]);

        let (res4, jq4) = self.solver.eval_diodepair(q[6], q[7], 1e-12, 1.28);

        self.jq[0] = jq1[0];
        self.jq[2] = jq2[0];
        self.jq[4] = jq3[0];
        self.jq[6] = jq4[0];

        self.solver.residue = [res1, res2, res3, res4];
    }

    #[inline(always)]
    fn solve_lin_equations(&mut self, b: [f64; N_N]) -> [f64; N_N] {
        let j00 = self.jq[0];
        let j11 = self.jq[2] * self.c1;
        let j12 = -self.jq[2] - 1.;
        let j22 = self.jq[4] * self.c1;
        let j23 = -self.jq[4] - 1.;
        // let j30 = -self.jq[6] + -3.;
        // let j32 = -self.jq[6] + -1. * self.c2;
        let j30 = -self.jq[6] - 4.;
        let j32 = -self.jq[6] - self.c2;
        let mut x = [0.; N_N];

        // x[0] = (((-b[0] + b[3]) * j12 - j32 * (b[0] * j11 + b[1])) * j23 + b[2] * j12
        //     - j22 * (b[0] * j11 + b[1]))
        //     / (((-j00 + j30) * j12 - j32 * j00 * j11) * j23 - j00 * j11 * j22);
        // x[1] = j00 * x[0] - b[0];
        // x[2] = (-j11 * x[1] + b[1]) / j12;
        // x[3] = j30 * x[0] + j32 * x[2] - b[3] - x[1];

        x[0] = (((-b[0] + b[3]) * j12 - j32 * (b[0] * j11 + b[1])) * j23 + 2. * b[2] * j12
            - 2. * j22 * (b[0] * j11 + b[1]))
            / (((j30 - j00) * j12 - j32 * j00 * j11) * j23 - 2. * j00 * j11 * j22);
        x[1] = j00 * x[0] - b[0];
        x[2] = (-j11 * x[1] + b[1]) / j12;
        x[3] = 0.5 * (j30 * x[0] + j32 * x[2] - b[3] - x[1]);
        x
    }
    pub fn reset(&mut self) {
        self.s = [0.; 2];
        self.solver.p_full = [0.; P_LEN];
        self.evaluate_nonlinearities([0.; N_N]);
        self.solver.set_extrapolation_origin([0.; N_P], [0.; N_N]);
    }
    // highpass and notch doesn't work right, likely because `input` isn't quite defined right. Prolly doesn't need to be subtracted?
    // ^ seems to be fixed now?
    fn get_output(&self, input: f32, k: f32) -> f32 {
        match self.params.mode {
            SvfMode::LP => self.vout[0],  // lowpass
            SvfMode::HP => self.vout[2],  // highpass
            SvfMode::BP1 => self.vout[1], // bandpass
            // the notch isn't limited to the -1 to 1 range like the other modes, not sure how to solve nicely for it currently
            SvfMode::Notch => input + k * self.vout[1], // notch
            //3 => input + 2. * k * self.vout[1], // allpass
            SvfMode::BP2 => k * self.vout[1], // bandpass (normalized peak gain)
                                              // _ => input + 2. * self.vout[1] + k * self.vout[0], // peak / resonator thingy
        }
    }
}
