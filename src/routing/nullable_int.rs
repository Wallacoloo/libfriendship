//! Implements a nullable integer. i.e. behaves similarly to
//! Option<NonZero<T>>
//! But freely serializes/deserializes/converts to T.
//! 

use num::Zero;
use serde::{Serialize, Serializer, Deserialize, Deserializer};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct NullableInt<T> {
    raw: T,
}

impl<T> NullableInt<T> {
    /// Relay NullableInt::new(x) to x.into() where
    /// x is convertible to NullableInt<T> (i.e. x = T or Option<T>)
    pub fn new<F>(raw: F) -> Self
        where F: Into<NullableInt<T>>
    {
        raw.into()
    }
}

impl<T: Copy + Zero> NullableInt<T> {
    /// Return None if self is null,
    /// else Some(value) where value != 0.
    pub fn get(&self) -> Option<T> {
        if self.raw.is_zero() {
            None
        } else {
            Some(self.raw)
        }
    }
    /// Cast self to T, where null is mapped to 0.
    pub fn get_raw(&self) -> T {
        self.raw
    }
}


/// T is directly convertable to NullableInt<T>
impl<T> From<T> for NullableInt<T> {
    fn from(raw: T) -> Self {
        Self{ raw }
    }
}
/// Option<T> is convertible to NullableInt:
/// None maps to null,
/// Some(0) maps to null
/// Some(x) maps to x
impl<T: Zero> From<Option<T>> for NullableInt<T> {
    fn from(maybe: Option<T>) -> Self {
        match maybe {
            None => Self{ raw: Zero::zero() },
            Some(raw) => Self{ raw }
        }
    }
}


impl<T: Zero + PartialEq + Copy> PartialEq<Option<T>> for NullableInt<T> {
    fn eq(&self, other: &Option<T>) -> bool {
        *self == Self::from(*other)
    }
    fn ne(&self, other: &Option<T>) -> bool {
        !self.eq(other)
    }
}

/// All NullableInts Serialize as if they were normal ints.
impl<T: Serialize> Serialize for NullableInt<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.raw.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for NullableInt<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let raw: T = Deserialize::deserialize(deserializer)?;
        Ok(Self{ raw })
    }
}
