extern crate num;
use self::num::complex::Complex32;

/// describes a sinusoidal of the form:
/// y = coeff * exp(i*ang_freq_t*t) * u(t - start_usec) * u(w-start_freq) * exp(i*ang_freq_w*w)
/// where u(t) is the unit step function,
/// and coeff is a complex exponential, which is used to encode both the
/// amplitude and phase shift of the sinusoid.
#[derive(Clone, Copy, Debug)]
pub struct Automation {
    /// time at which the automation should be gated on, in microseconds
    start_usec : u64,
    /// complex amplitude coefficient
    coeff : Complex32,
    /// frequency of the sinusoid in the dimension of time, in radians/second
    ang_freq_t : f32,
    /// the w at which the automation should be gated on, in radians/sec
    start_freq : f32,
    /// frequency of the sinusoid in the dimension of w, in radians/(radians/sec)
    ang_freq_w : f32,
}

impl Automation {
    pub fn new(start_usec : u64, coeff : Complex32, ang_freq_t : f32,
      start_freq : f32, ang_freq_w : f32) -> Automation {
        Automation{
            start_usec: start_usec,
            coeff: coeff,
            ang_freq_t: ang_freq_t,
            start_freq: start_freq,
            ang_freq_w: ang_freq_w,
        }
    }
    pub fn start_time(&self) -> u64 {
        self.start_usec
    }
    pub fn coeff(&self) -> Complex32 {
        self.coeff
    }
    pub fn ang_freq_t(&self) -> f32 {
        self.ang_freq_t
    }
    pub fn start_freq(&self) -> f32 {
        self.start_freq
    }
    pub fn ang_freq_w(&self) -> f32 {
        self.ang_freq_w
    }
}