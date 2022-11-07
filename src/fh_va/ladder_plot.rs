// in case you want to plot the ladder's transfer function

fn get_ladder_bode(
    cutoff: f32,
    k: f32,
    ladder_mix: [f32; 5],
    len: usize,
) -> Vec<Complex<f32>> {
    let g = cutoff;
    let mut frequencies = vec![1.; len];
    let mut array = vec![Complex::new(0., 0.); len];
    // frequency map setup
    const MIN: f32 = 5.;
    const MAX: f32 = 20000.;
    let min_log: f32 = MIN.log2();
    let range: f32 = MAX.log2() - min_log;
    for i in 0..len {
        frequencies[i] = 2.0f32.powf(((i as f32 / len as f32) * range) + min_log);
    }
    let j = Complex::new(0., 1.);
    let mut curr_s: Complex<f32>;
    
    for i in 0..len {
        curr_s = frequencies[i] * j;
        for idx in 0..5 {
            array[i] += bjt_ladder_base(curr_s, g, k, idx as i32 - 1) * ladder_mix[idx];
        }
    }
        
    return array;
}

// all the poles follow this transfer, with pole == 0 for the feedback, pole == 1 for pole 1 etc.
// when just adding them together according to the pole mix we get the right transfer function
fn bjt_ladder_base(s: Complex<f32>, g: f32, k: f32, pole: i32) -> Complex<f32> {
    ((1. + s / g).powi(4 - slope as i32)) / (k + (1. + s / g).powi(4))
}

pub fn get_amplitude_response(
    cutoff: f32,
    k: f32,
    ladder_mix: [f32; 5],
    len: usize,
) -> Vec<f32> {
    let array = get_ladder_bode(cutoff, k, mode, filter_type, ladder_mix, len);
    let mut amplitudes = vec![1.; len];
    for i in 0..len {
        amplitudes[i] = lin_to_db(array[i].norm());
    }

    amplitudes
}
// phases are in range -PI to PI
pub fn get_phase_response(
    cutoff: f32,
    k: f32,
    mode: usize,
    filter_type: Circuits,
    ladder_mix: [f32; 5],
    len: usize,
) -> Vec<f32> {
    let array = get_filter_bode(cutoff, k, mode, filter_type, ladder_mix, len);
    let mut phases = vec![1.; len];
    for i in 0..len {
        phases[i] = array[i].arg();
    }
    phases
}