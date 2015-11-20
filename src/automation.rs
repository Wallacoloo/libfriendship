extern crate num;

use partial::Partial;
use phaser::PhaserCoeff;
use real::Real32;

/// describes an automation of the form:
/// y = coeff * exp(i*wt*t) * exp(i*ww*w)
/// where coeff is a complex exponential, which is used to encode both the
/// amplitude and phase shift of the sinusoid.
/// wt (omega) is the frequency of the automation,
/// and ww (omega_w) is the modulation parameter.
/// w is substituted with the partial's wt upon multiplication.
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub struct Automation {
    /// complex amplitude coefficient
    coeff : PhaserCoeff,
    /// frequency of the sinusoid, in radians/second
    omega : Real32,
    /// modulation frequency (used in binary operations)
    omega_w : Real32,
}

impl Automation {
    pub fn new(coeff : PhaserCoeff, omega : Real32, omega_w : Real32) -> Automation {
        Automation{
            coeff: coeff,
            omega: omega,
            omega_w: omega_w
        }
    }
    pub fn coeff(&self) -> PhaserCoeff {
        self.coeff
    }
    pub fn omega(&self) -> Real32 {
        self.omega
    }
    pub fn omega_w(&self) -> Real32 {
        self.omega_w
    }
    /// "multiply" the partial with this automation.
    pub fn apply_to_partial(&self, other: Partial) -> Partial {
        unimplemented!();
    }
    /// "multiply" the two automations
    pub fn apply_to_autom(&self, other: Automation) -> Automation {
        unimplemented!();
    }
}

