extern crate num;
use self::num::complex::Complex32;

/// describes an automation of the form:
/// y = coeff * exp(i*wt*t) * exp(i*ww*w)
/// where coeff is a complex exponential, which is used to encode both the
/// amplitude and phase shift of the sinusoid.
/// wt (omega) is the frequency of the automation,
/// and ww (omega_w) is the modulation parameter.
/// w is substituted with the partial's wt upon multiplication.
#[derive(Clone, Copy, Debug)]
pub struct Automation {
    /// complex amplitude coefficient
    coeff : Complex32,
    /// frequency of the sinusoid, in radians/second
    omega : f32,
    /// modulation frequency (used in binary operations)
    omega_w : f32,
}

impl Automation {
    pub fn new(coeff : Complex32, omega : f32, omega_w : f32) -> Automation {
        Automation{
            coeff: coeff,
            omega: omega,
            omega_w: omega_w
        }
    }
    pub fn coeff(&self) -> Complex32 {
        self.coeff
    }
    pub fn omega(&self) -> f32 {
        self.omega
    }
    pub fn omega_w(&self) -> f32 {
        self.omega_w
    }
}