extern crate num;
use self::num::complex::Complex32;

/// describes a sinusoidal of the form:
/// y = coeff * exp(i*ang_freq*t) * u(t - start_usec)
/// where u(t) is the unit step function,
/// and coeff is a complex exponential, which is used to encode both the
/// amplitude and phase shift of the sinusoid.
#[derive(Clone, Copy, Debug)]
pub struct Partial {
    /// time at which the partial should be gated on, in microseconds
    start_usec : u64,
    /// complex amplitude coefficient
    coeff : Complex32,
    /// frequency of the sinusoid, in radians/second
    ang_freq : f32,
    /// index of the stereo/surround channel that this wave occupies
    /// e.g. 0 for left, 1 for right.
    channel : u8,
}

impl Partial {
    pub fn new(start_usec : u64, coeff : Complex32, ang_freq : f32, channel : u8) -> Partial {
        Partial{
            start_usec: start_usec,
            coeff: coeff,
            ang_freq: ang_freq,
            channel: channel
        }
    }
    pub fn start_time(&self) -> u64 {
        self.start_usec
    }
    pub fn coeff(&self) -> Complex32 {
        self.coeff
    }
    pub fn ang_freq(&self) -> f32 {
        self.ang_freq
    }
    pub fn channel(&self) -> u8 {
        self.channel
    }
}