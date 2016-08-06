extern crate num_complex;

use std::fmt;

use self::num_complex::Complex32;

use tree::node::NodeOp;

/// describes a signal of the form:
/// y = [c cos(w*t-phase) u(t-start) u'(t-end), a]
/// Where `a` is some "modulation parameter", which serves only to effect
/// other signals when `y` is involved in a binary op.
/// u(t) is the unit step: u(t) = 1 if t > 0, 0 if t < 0
/// u'(t) = 1-u(t)
#[derive(Clone, Copy, Debug)]
pub struct Signal {
    /// Amplitude
    c : f32,
    /// frequency of the sinusoid, in radians/second
    w : f32,
    /// phase
    phase : f32,
    /// modulation parameter (used in binary operations)
    a : f32,
    /// signal start time (seconds)
    start: f32,
    /// signal end time (seconds)
    end: f32,
}

impl Signal {
    pub fn new(c : f32, w : f32, phase : f32, a : f32, start : f32, end : f32) -> Signal {
        Signal{
            c: c,
            w: w,
            phase: phase,
            a: a,
            start: start,
            end: end,
        }
    }
    pub fn amp(&self) -> f32 {
        self.c
    }
    pub fn phase(&self) -> f32 {
        self.phase
    }
    pub fn ang_freq(&self) -> f32 {
        self.w
    }
    pub fn parameter(&self) -> f32 {
        self.a
    }
    pub fn start(&self) -> f32 {
        self.start
    }
    pub fn end(&self) -> f32 {
        self.end
    }
    pub fn phaser_coeff(&self) -> Complex32 {
        // y = 0.5*c[exp(j*w*t-j*phase) + exp(-j*w*t+j*phase)]
        // so the complex coefficient is 0.5*c*exp(-j*phase)
        Complex32::from_polar(&(0.5*self.c), &-self.phase)
    }
    /// Perform arithmetic operation: self <op> other.
    /// Note `self` is on the left.
    pub fn apply_on_left(&self, other: &Signal, op: &NodeOp) -> [Signal; 2] {
        let c1 = self.c;
        let c2 = other.c;
        let w1 = self.w;
        let w2 = other.w;
        let phase1 = self.phase;
        let phase2 = other.phase;
        let a1 = self.a;
        let a2 = other.a;
        let start1 = self.start;
        let start2 = other.start;
        let end1 = self.end;
        let end2 = other.end;

        let amp = 0.5*c1*c2; // amplitude of each of the two output cosines.

        match op {
            &NodeOp::OpAt => {
                // This represents a sort of delay, or "applying" y2 to y1.
                // Essentially, y1(t-a2)*y2(t)
                // c1*cos(w1*(t-a2)-phase1) u(t-a2-start1)u'(t-a2-end1) * c2*cos(w2*t-phase2)
                //   u(t-start2) u'(t-end2)
                // = 0.5*c1*c2*cos[(w1+w2)*t - (phase1+phase2+a2*w1)]*gate(t)
                // + 0.5*c1*c2*cos[(w1-w2)*t - (phase1-phase2+a2*w1)]*gate(t)
                // The new waves are given a1's modulation parameter.
                let start1 = start1 + a2;
                let end1 = end1 + a2;
                let start = start1.max(start2);
                let end = end1.min(end2);
                [Signal::new(amp, w1+w2, phase1+a2*w1+phase2, a1, start, end),
                 Signal::new(amp, w1-w2, phase1+a2*w1-phase2, a1, start, end)]
            },
            &NodeOp::OpBy => {
                // This represents "multiplication" of two waves, suitable
                // specifically for automations. y(t) = y1(t)*y2(t), a=a1-a2
                let start = start1.max(start2);
                let end = end1.min(end2);
                [Signal::new(amp, w1+w2, phase1+phase2, a1-a2, start, end),
                 Signal::new(amp, w1-w2, phase1-phase2, a1-a2, start, end)]
            },
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{} cos({}*t - {}) gate[{}, {}](t), a={}]", self.amp(), self.ang_freq(), self.phase(), self.start(), self.end(), self.parameter())
    }
}

