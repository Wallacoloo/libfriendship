use std::fmt;

use phaser::PhaserCoeff;
use real::Real32;

use tree::node::NodeOp;

/// describes a signal of the form:
/// y = [c cos(w*t-phase), a]
/// Where `a` is some "modulation parameter", which serves only to effect
/// other signals when `y` is involved in a binary op.
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub struct Signal {
    /// Amplitude
    c : Real32,
    /// frequency of the sinusoid, in radians/second
    w : Real32,
    /// phase
    phase : Real32,
    /// modulation parameter (used in binary operations)
    a : Real32,
    /// signal start time (seconds)
    start: Real32,
    /// signal end time (seconds)
    end: Real32,
}

impl Signal {
    pub fn new(c : Real32, w : Real32, phase : Real32, a : Real32, start : Real32, end : Real32) -> Signal {
        Signal{
            c: c,
            w: w,
            phase: phase,
            a: a,
            start: start,
            end: end,
        }
    }
    pub fn new_f(c : f32, w : f32, phase : f32, a : f32, start : f32, end : f32) -> Signal {
        Signal::new(Real32::new(c), Real32::new(w), Real32::new(phase), Real32::new(a), Real32::new(start), Real32::new(end))
    }
    pub fn amp(&self) -> Real32 {
        self.c
    }
    pub fn phase(&self) -> Real32 {
        self.phase
    }
    pub fn ang_freq(&self) -> Real32 {
        self.w
    }
    pub fn parameter(&self) -> Real32 {
        self.a
    }
    pub fn start(&self) -> Real32 {
        self.start
    }
    pub fn end(&self) -> Real32 {
        self.end
    }
    pub fn phaser_coeff(&self) -> PhaserCoeff {
        // y = 0.5*c[exp(j*w*t-j*phase) + exp(-j*w*t+j*phase)]
        // so the complex coefficient is 0.5*c*exp(-j*phase)
        PhaserCoeff::new(Real32::new(0.5)*self.c, Real32::new(0.0))*self.phase.expj().conj()
    }
    pub fn apply_to_left(&self, other: &Signal, op: &NodeOp) -> Signal {
        // TODO: implement
        //Signal::new(self.c, self.w, self.a, self.start)
        //
        // Let a1 = exp(c1 + j w1 t + j a1 w) + exp(-c1 - j w1 t - j a1 w)
        // Let a2 = exp(c2 + j w2 t + j a2 w) + exp(-c2 - j w2 t - j a2 w)
        //let c1 = self.c;
        //let w1 = self.w;
        //let a1 = self.a;
        //let c2 = other.c;
        //let w2 = other.w;
        //let a2 = other.a;
        //let start = self.start.max(other.start);
        //match op {
        //    &NodeOp::OpMul => {
        //        // a1 * a2:
        //        // = exp(c1+c2 + j*t(w1+w2) + j*w(a1+a2)) + exp(c1-c2 + j*t(w1-w2) + j*w(a1-a2))
        //        // + exp(c1*+c2* + j*t(-w1-w2) + j*w(-a1-a2)) + exp(c2-c1 + j*t(w2-w1) + j*w(a2-a1))
        //        // The above are combined into two pairs of complex conjugates,
        //        // meaning two signals.
        //        let sig1 = Signal::new(c1+c2, w1+w2, a1+a2, start);
        //        let sig2 = Signal::new((c1+c2).conj(), w1-w2, a1-a2, start);
        //        sig2
        //    },
        //    &NodeOp::OpAt => {
        //        // a1 @ a2:
        //        // = exp[c1+c2+j*a2*w1 + j*t(w1+w2) + j*w(a1)] + exp[c1-c2-j*a1*w1 + j*t(w1-w2) +
        //        // j*w(a1)]
        //        // + exp[-c1-c2
        //        Signal::new(self.c, self.w, self.a, self.start)
        //    },
        //    &NodeOp::OpBy => {
        //        Signal::new(self.c, self.w, self.a, self.start)
        //    },
        //}
        Signal::new_f(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} cos({}*t - {}) gate[{}, {}](t), a={}]", self.amp(), self.ang_freq(), self.phase(), self.start(), self.end(), self.parameter())
    }
}

