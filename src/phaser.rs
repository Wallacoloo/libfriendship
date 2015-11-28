use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

use real::Real32;


/// Normal Complex32 numbers are only PartialEq
/// because f32::NaN != f32::NaN (gah!)
/// This implements a subset of complex numbers that could be reasonably used to
/// control the phase/amplitude of a sinusoid.
/// Specifically, both parts of the number are finite.

#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(Hash)]
pub struct PhaserCoeff {
    re: Real32,
    im: Real32,
}

impl PhaserCoeff {
    pub fn new(re: Real32, im: Real32) -> PhaserCoeff {
        PhaserCoeff{ re: re, im: im }
    }
    pub fn new_f32(re: f32, im: f32) -> PhaserCoeff {
        PhaserCoeff::new(Real32::new(re), Real32::new(im))
    }
    pub fn re(&self) -> Real32 {
        self.re
    }
    pub fn im(&self) -> Real32 {
        self.im
    }
    pub fn norm_sqr(&self) -> Real32 {
        self.re()*self.re() + self.im()*self.im()
    }
    
    /// return exp(i*value), where i^2 = -1
    /// # Examples
    ///
    /// e^(i*pi) = -1
    ///
    /// ```
    /// use std::f32;
    /// use libfriendship::phaser::PhaserCoeff;
    /// use libfriendship::real::Real32;
    ///
    /// let expected = PhaserCoeff::new_f32(-1f32, 0f32);
    /// let got = PhaserCoeff::expi(Real32::new(f32::consts::PI));
    /// assert!((expected-got).norm_sqr().value() < 0.0000001f32);
    /// ```
    pub fn expi(value: Real32) -> PhaserCoeff {
        // expi(w) = cos(w) + i*sin(w)
        let sc = value.sin_cos();
        PhaserCoeff::new(sc.1, sc.0)
    }
}

impl fmt::Display for PhaserCoeff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} + {}i)", self.re(), self.im())
    }
}


// Macros taken from the num::complex lib.
// These allow &PhaserCoeff * &PhaserCoeff,
//           or PhaseCoeff  *  PhaseCoeff,
//           or any other combination, for all binary ops.
macro_rules! forward_val_val_binop {
    (impl $imp:ident, $method:ident) => {
        impl $imp<PhaserCoeff> for PhaserCoeff {
            type Output = PhaserCoeff;

            #[inline]
            fn $method(self, other: PhaserCoeff) -> PhaserCoeff {
                (&self).$method(&other)
            }
        }
    }
}

macro_rules! forward_ref_val_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a> $imp<PhaserCoeff> for &'a PhaserCoeff {
            type Output = PhaserCoeff;

            #[inline]
            fn $method(self, other: PhaserCoeff) -> PhaserCoeff {
                self.$method(&other)
            }
        }
    }
}

macro_rules! forward_val_ref_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a> $imp<&'a PhaserCoeff> for PhaserCoeff {
            type Output = PhaserCoeff;

            #[inline]
            fn $method(self, other: &PhaserCoeff) -> PhaserCoeff {
                (&self).$method(other)
            }
        }
    }
}

macro_rules! forward_all_binop {
    (impl $imp:ident, $method:ident) => {
        forward_val_val_binop!(impl $imp, $method);
        forward_ref_val_binop!(impl $imp, $method);
        forward_val_ref_binop!(impl $imp, $method);
    };
}


forward_all_binop!(impl Add, add);

impl<'a, 'b> Add<&'a PhaserCoeff> for &'b PhaserCoeff {
    type Output = PhaserCoeff;

    fn add(self, other: &PhaserCoeff) -> PhaserCoeff {
        PhaserCoeff::new(self.re() + other.re(), self.im() + other.im())
    }
}

// Note: Div cannot be applied to a PhaserCoeff,
//   as not all complex numbers have an inverse.

impl Neg for PhaserCoeff {
    type Output = PhaserCoeff;

    fn neg(self) -> PhaserCoeff {
        PhaserCoeff::new(-self.re(), -self.im())
    }
}

forward_all_binop!(impl Mul, mul);

impl<'a, 'b> Mul<&'a PhaserCoeff> for &'b PhaserCoeff {
    type Output = PhaserCoeff;
    
    /// # Examples
    /// ```
    /// use libfriendship::phaser::PhaserCoeff;
    /// let a = PhaserCoeff::new_f32(1f32, 2f32);
    /// let b = PhaserCoeff::new_f32(3f32, 4f32);
    /// let c = a * b;
    /// assert_eq!(c, PhaserCoeff::new_f32(-5f32, 10f32));
    /// ```
    fn mul(self, other: &PhaserCoeff) -> PhaserCoeff {
        //  (a + bi)(c + di)
        //= ac + adi + bci - bd
        //= (ac - bd) + (ad + bc)i
        let a = self.re();
        let b = self.im();
        let c = other.re();
        let d = other.im();
        PhaserCoeff::new(a*c - b*d, a*d + b*c)
    }
}

forward_all_binop!(impl Sub, sub);

impl<'a, 'b> Sub<&'a PhaserCoeff> for &'b PhaserCoeff {
    type Output = PhaserCoeff;
    
    fn sub(self, other: &PhaserCoeff) -> PhaserCoeff {
        PhaserCoeff::new(self.re() - other.re(), self.im() - other.im())
    }
}
