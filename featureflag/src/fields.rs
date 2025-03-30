use std::fmt;

use crate::value::Value;

#[derive(Clone)]
pub struct Fields<'a> {
    fields: &'a [(&'a str, Value<'a>)],
}

impl<'a> Fields<'a> {
    pub fn new(fields: &'a [(&'a str, Value<'a>)]) -> Fields<'a> {
        Fields { fields }
    }

    pub fn values(&self) -> impl '_ + Iterator<Item = (&'a str, &'a Value<'a>)> {
        self.fields.iter().map(|(k, v)| (*k, v))
    }

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
