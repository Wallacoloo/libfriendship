use std::fmt;

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
    /// Given Y = c1 exp(i*wt1),
    /// A = c2 exp(i*wt2) exp(i*ww2*wt1)
    /// Then the result of A*Y is:
    /// Yout = c1 c2 exp(i*wt1) exp(i*wt2) exp(i*ww2*wt1)
    ///
    /// # Example
    /// ```
    /// use std::f32;
    /// use libfriendship::automation::Automation;
    /// use libfriendship::partial::Partial;
    /// use libfriendship::phaser::PhaserCoeff;
    /// use libfriendship::real::Real32;
    ///
    /// // create a 500 rad/sec *sine* wave
    /// let p = Partial::new(PhaserCoeff::new_f32(0f32, -1f32), Real32::new(500.25));
    /// let a = Automation::new(PhaserCoeff::new_f32(0.5f32, 0f32), Real32::new(100.0),
    /// Real32::new(2f32*f32::consts::PI));
    /// let m = a.apply_to_partial(p);
    /// // we expect -i*0.5*expi(500.25 t)*expi(100 t)*expi(500.25*2 pi)
    /// // = -i*0.5*i*expi((500.25+100) t)
    /// // = 0.5*expi(600.25 t)
    /// let expected = Partial::new(PhaserCoeff::new_f32(0.5, 0f32), Real32::new(600.25));
    /// println!("got: {}, expected {}", m, expected);
    /// assert!((expected.coeff() - m.coeff()).norm_sqr().value() < 0.0000001f32);
    /// assert!((expected.ang_freq() - m.ang_freq()).value().abs() < 0.00001f32);
    ///
    /// ```
    pub fn apply_to_partial(&self, other: Partial) -> Partial {
        let phase_shift = PhaserCoeff::expi(self.omega_w()*other.ang_freq());
        let coeff = other.coeff()*self.coeff() * phase_shift;
        let omega = self.omega() + other.ang_freq();
        Partial::new(coeff, omega)
    }
    /// "multiply" the two automations
    /// Given A1 = c1 exp(i*wt1) exp(i*ww1*wtp)
    /// A2 = c2 exp(i*wt2) exp(i*ww2*ww1)
    /// Then the result of A1*A2 (A2.apply_to_autom(A1)) is:
    /// Aout = c1 c2 exp(i*wt1) exp(i*wt2) exp(i*ww2*ww1) exp(i*ww1*wtp)
    pub fn apply_to_autom(&self, other: Automation) -> Automation {
        let coeff = other.coeff()*self.coeff() * PhaserCoeff::expi(self.omega_w()*other.omega_w());
        let omega = self.omega() + other.omega();
        let omega_w = other.omega_w();
        Automation::new(coeff, omega, omega_w)
    }
}

impl fmt::Display for Automation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} expi({} t) expi({} wt)", self.coeff(), self.omega(), self.omega_w())
    }
}

