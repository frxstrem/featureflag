//! Value types for the [`context!`](macro@crate::context) macro.

use std::{borrow::Cow, fmt};

/// A value that can be passed as a field in a [`context!`](macro@crate::context).
#[derive(Clone, Default)]
pub enum Value<'a> {
    /// A string value.
    Str(Cow<'a, str>),

    /// A byte array value.
    Bytes(Cow<'a, [u8]>),

    /// A boolean value.
    Bool(bool),

    /// A signed 64-bit integer value.
    I64(i64),

    /// An unsigned 64-bit integer value.
    U64(u64),

    /// A 64-bit floating-point value.
    F64(f64),

    /// A null value.
    #[default]
    Null,
}

impl Value<'_> {
    /// Clone a new `Value` with a `'static` lifetime.
    pub fn to_static(&self) -> Value<'static> {
        match self {
            Value::Str(s) => Value::Str(Cow::Owned(s.clone().into_owned())),
            Value::Bytes(b) => Value::Bytes(Cow::Owned(b.clone().into_owned())),
            Value::Bool(b) => Value::Bool(*b),
            Value::U64(n) => Value::U64(*n),
            Value::I64(n) => Value::I64(*n),
            Value::F64(x) => Value::F64(*x),
            Value::Null => Value::Null,
        }
    }

    /// Convert a `Value` to a `'static` lifetime, consuming the original value.
    pub fn into_static(self) -> Value<'static> {
        match self {
            Value::Str(s) => Value::Str(Cow::Owned(s.into_owned())),
            Value::Bytes(b) => Value::Bytes(Cow::Owned(b.into_owned())),
            Value::Bool(b) => Value::Bool(b),
            Value::U64(n) => Value::U64(n),
            Value::I64(n) => Value::I64(n),
            Value::F64(x) => Value::F64(x),
            Value::Null => Value::Null,
        }
    }

    /// Get the value as a string, if it is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Get the value as a byte slice, if it is a byte slice.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// Get the value as a boolean, if it is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the value as a signed 64-bit integer, if it is a signed 64-bit integer.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(n) => Some(*n),
            _ => None,
        }
    }

    /// Get the value as an unsigned 64-bit integer, if it is an unsigned 64-bit integer.
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::U64(n) => Some(*n),
            _ => None,
        }
    }

    /// Get the value as a 64-bit floating-point number, if it is a 64-bit floating-point number.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(x) => Some(*x),
            _ => None,
        }
    }

    /// Check if the value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl fmt::Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "{:?}", s),
            Value::Bytes(b) => write!(f, "{:?}", b),
            Value::Bool(b) => write!(f, "{:?}", b),
            Value::I64(n) => write!(f, "{:?}", n),
            Value::U64(n) => write!(f, "{:?}", n),
            Value::F64(x) => write!(f, "{:?}", x),
            Value::Null => write!(f, "null"),
        }
    }
}

/// A trait for types that can be converted to a [`Value`].
pub trait ToValue {
    /// Convert the type to a [`Value`].
    fn to_value(&self) -> Value<'_>;
}

impl<V: ToValue> ToValue for Option<V> {
    fn to_value(&self) -> Value<'_> {
        match self {
            Some(v) => v.to_value(),
            None => Value::Null,
        }
    }
}

impl<V: ?Sized + ToValue> ToValue for &'_ V {
    fn to_value(&self) -> Value<'_> {
        V::to_value(self)
    }
}

impl<V: ToValue + Clone> ToValue for Cow<'_, V> {
    fn to_value(&self) -> Value<'_> {
        self.as_ref().to_value()
    }
}

impl ToValue for str {
    fn to_value(&self) -> Value<'_> {
        Value::Str(Cow::Borrowed(self))
    }
}

impl ToValue for String {
    fn to_value(&self) -> Value<'_> {
        Value::Str(Cow::Borrowed(self))
    }
}

impl ToValue for [u8] {
    fn to_value(&self) -> Value<'_> {
        Value::Bytes(Cow::Borrowed(self))
    }
}

impl ToValue for Vec<u8> {
    fn to_value(&self) -> Value<'_> {
        Value::Bytes(Cow::Borrowed(self))
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value<'_> {
        Value::Bool(*self)
    }
}

impl ToValue for i8 {
    fn to_value(&self) -> Value<'_> {
        Value::I64(i64::from(*self))
    }
}

impl ToValue for i16 {
    fn to_value(&self) -> Value<'_> {
        Value::I64(i64::from(*self))
    }
}

impl ToValue for i32 {
    fn to_value(&self) -> Value<'_> {
        Value::I64(i64::from(*self))
    }
}

impl ToValue for i64 {
    fn to_value(&self) -> Value<'_> {
        Value::I64(*self)
    }
}

impl ToValue for u8 {
    fn to_value(&self) -> Value<'_> {
        Value::U64(u64::from(*self))
    }
}

impl ToValue for u16 {
    fn to_value(&self) -> Value<'_> {
        Value::U64(u64::from(*self))
    }
}

impl ToValue for u32 {
    fn to_value(&self) -> Value<'_> {
        Value::U64(u64::from(*self))
    }
}

impl ToValue for u64 {
    fn to_value(&self) -> Value<'_> {
        Value::U64(*self)
    }
}

impl ToValue for f32 {
    fn to_value(&self) -> Value<'_> {
        Value::F64(f64::from(*self))
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value<'_> {
        Value::F64(*self)
    }
}
