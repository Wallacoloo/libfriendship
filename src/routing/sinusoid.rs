extern crate num;

use self::num::complex::{Complex32, Complex64};
type C32 = Complex32;
type C64 = Complex64;

#[derive(Clone)]
pub struct Sinusoid {
    /// How much to increment the phase per sample. Has the same dimension as "frequency", but
    /// different units.
    phase_delta: f32,
    /// Phase at t=0
    phase_0: f32,
}

/// Structure to evaluate a sinusoid at successive points.
// to precisely evaluate the sinusoid, we note:
// sin(x) = e^{j*x}.Im
// sin(w0+d*t) = {e^{j*w0} * e^{j*w*t}}.Im
// if w is the phase_delta, and t is the integer sample index, then w*t can be reliably computed
//   for any sane value of w, t using f64.
//   Therefore, e^{j*w*t} is also accurate, based on how the sin/cos function work
//   (they use a more precise reduction than just fmod(w*t, TWO_PI)).
// The same logic applies for e^{j*w0} (phase_0 is sane).
// We split the operation this way to avoid adding floats with wildly different magnitudes
// when t is large.
pub struct SinusoidIter<'a> {
    sin: &'a Sinusoid,
    /// e^{j*phase_0}
    value_0 : C32,
    idx: u32,
}


impl<'a> Sinusoid {
    pub fn get_consec(&'a self, offset: u32) -> SinusoidIter<'a> {
        SinusoidIter {
            sin: &self,
            value_0: C32::new(0f32, self.phase_0).exp(),
            idx: 0,
        }
    }
}

impl<'a> Iterator for SinusoidIter<'a> {
    type Item=f32;
    fn next(&mut self) -> Option<f32> {
        // Solve for e^{j*w*t}
        let phase = (self.idx as f64)*(self.sin.phase_delta as f64);
        let value_delta = C64::new(0f64, phase).exp();
        // multiply by the pre-solved e^{j*w0}
        let value_delta = C32::new(value_delta.re as f32, value_delta.im as f32);
        let value = (value_delta * self.value_0).im;
        // Prepare for next iter & return.
        self.idx += 1;
        Some(value)
    }
}
