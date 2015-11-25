use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Add, Mul, Neg, Sub};


/// Like f32, but has the additional restriction that the value is *finite*
/// and never NaN.
/// This makes a Real satisfy Total Equality, whereas normal f32 is only
/// PartialEq due to NaNs.
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, PartialOrd)]
pub struct Real32 {
	value: f32,
}

impl Real32 {
    /// It should be possible to create a Real from any finite float.
    /// 
    /// # Failure Examples:
    ///
    /// ```should_panic
    /// use libfriendship::real::Real32;
    /// println!("Real32({})", 0.0f32 / 0.0f32);
    /// Real32::new(0.0f32 / 0.0f32);
    /// ```
    ///
    /// ```should_panic
    /// use libfriendship::real::Real32;
    /// println!("Real32({})", 1.0f32 / 0.0f32);
    /// Real32::new(1.0f32 / 0.0f32);
    /// ```
    pub fn new(value: f32) -> Real32 {
        assert!(value.is_finite());
        Real32{ value: value }
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    
    /// return (sin(self), cos(self))
    pub fn sin_cos(&self) -> (Real32, Real32) {
        let sc = self.value().sin_cos();
        (Real32::new(sc.0), Real32::new(sc.1))
    }
}


impl Eq for Real32 {}

impl Hash for Real32 {
    /// Any values that are equal should have the same hash.
    /// 
    /// # Examples:
    ///
    /// ```
    /// use libfriendship::real::Real32;
    /// use std::hash::{Hash, Hasher, SipHasher};
    ///
    /// let mut hash_pos = SipHasher::new();
    /// let mut hash_neg = SipHasher::new();
    /// Real32::new(0.0f32).hash(&mut hash_pos);
    /// Real32::new(-0.0f32).hash(&mut hash_neg);
    /// assert_eq!(hash_pos.finish(), hash_neg.finish());
    /// ```
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        // we need to satisfy hash(x) = hash(y) for any x=y.
        // This includes +0.0 == -0.0,
        // so before hashing the bits, force the number to be non-negative:
        let v = self.value.abs();
        // reinterpret the bytes of our float as a u32 & hash that
        state.write_u32(unsafe {
            mem::transmute(v)
        });
    }
}


// Arithmetic traits

impl Add for Real32 {
    type Output = Real32;

    fn add(self, rhs: Real32) -> Real32 {
        Real32::new(self.value() + rhs.value())
    }
}

// Div cannot be safely applied to Reals, as R / 0 doesn't yield a Real
/*impl Div for Real32 {
    type Output = Real32;

    fn div(self, rhs: Real32) -> Real32 {
        Real32::new(self.value() / rhs.value())
    }
}*/

impl Mul for Real32 {
    type Output = Real32;

    fn mul(self, rhs: Real32) -> Real32 {
        Real32::new(self.value() * rhs.value())
    }
}

impl Neg for Real32 {
    type Output = Real32;

    fn neg(self) -> Real32 {
        Real32::new(-self.value())
    }
}

impl Sub for Real32 {
    type Output = Real32;

    fn sub(self, rhs: Real32) -> Real32 {
        Real32::new(self.value() - rhs.value())
    }
}
