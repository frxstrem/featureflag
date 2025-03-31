//! `Fields` struct and macro for creating a collection of fields for use
//! in [`context!`](macro@crate::context).

use std::fmt;

use crate::value::Value;

/// A struct representing a collection of fields.
///
/// This struct is constructed using the `fields!` macro.
#[derive(Clone)]
pub struct Fields<'a> {
    fields: &'a [(&'a str, Value<'a>)],
}

impl<'a> Fields<'a> {
    /// Creates a new `Fields` instance with the given fields.
    pub fn new(fields: &'a [(&'a str, Value<'a>)]) -> Fields<'a> {
        Fields { fields }
    }

    /// Iterate over the fields in the `Fields` struct.
    pub fn pairs(&self) -> impl '_ + Iterator<Item = (&'a str, &'a Value<'a>)> {
        self.fields.iter().map(|(k, v)| (*k, v))
    }

    /// Get a field by its key.
    pub fn get(&self, key: &str) -> Option<&'a Value<'a>> {
        self.fields.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }
}

impl fmt::Debug for Fields<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.fields.iter().map(|(k, v)| (k, v)))
            .finish()
    }
}

/// Creates a new `Fields` instance with the given fields.
///
/// The fields are specified as a comma-separated list of `key = value` pairs.
/// Field values can be any type that implements the [`ToValue`](crate::value::ToValue) trait.
#[macro_export]
macro_rules! fields {
    (@__entry $key:ident) => { (stringify!($key), $crate::value::ToValue::to_value(&$key)) };
    (@__entry $key:ident = $expr:expr) => { (stringify!($key), $crate::value::ToValue::to_value(&$expr)) };
    (@__entry $key:literal = $expr:expr) => { ($key, $crate::value::ToValue::to_value(&$expr)) };
    (@__entry [$key:expr] = $expr:expr) => { (&$key as &str, $crate::value::ToValue::to_value(&$expr)) };

    () => {
        $crate::fields::Fields::new(&[])
    };

    ( $(
        $name:tt $(= $expr:expr)?
    ),+ $(,)? ) => {
        $crate::fields::Fields::new(&[
            $(
                $crate::fields!(@__entry $name $(= $expr)?),
            )*
        ])
    };
}
