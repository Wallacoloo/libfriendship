use std::cmp::{PartialEq, PartialOrd};
use std::hash::Hash;

use phaser::PhaserCoeff;
use real::Real32;

/// describes a sinusoidal of the form:
/// y = coeff * exp(i*ang_freq*t)
/// where coeff is a complex exponential, which is used to encode both the
/// amplitude and phase shift of the sinusoid.
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub struct Partial {
    /// complex amplitude coefficient
    coeff : PhaserCoeff,
    /// frequency of the sinusoid, in radians/second
    ang_freq : Real32,
}

impl Partial {
    pub fn new(coeff : PhaserCoeff, ang_freq : Real32) -> Partial {
        Partial{
            coeff: coeff,
            ang_freq: ang_freq,
        }
    }
    pub fn coeff(&self) -> PhaserCoeff {
        self.coeff
    }
    pub fn ang_freq(&self) -> Real32 {
        self.ang_freq
    }
}
