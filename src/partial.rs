extern crate num;

use std::cmp::{Ord, Ordering};
use std::hash::Hash;

use self::num::complex::Complex32;

/// describes a sinusoidal of the form:
/// y = coeff * exp(i*ang_freq*t)
/// where coeff is a complex exponential, which is used to encode both the
/// amplitude and phase shift of the sinusoid.
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Hash)]
pub struct Partial {
    /// complex amplitude coefficient
    coeff : Complex32,
    /// frequency of the sinusoid, in radians/second
    ang_freq : f32,
}

impl Partial {
    pub fn new(coeff : Complex32, ang_freq : f32) -> Partial {
        Partial{
            coeff: coeff,
            ang_freq: ang_freq,
        }
    }
    pub fn coeff(&self) -> Complex32 {
        self.coeff
    }
    pub fn ang_freq(&self) -> f32 {
        self.ang_freq
    }
}
