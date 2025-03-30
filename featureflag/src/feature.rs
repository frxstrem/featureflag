#[cfg(feature = "feature-registry")]
use std::{collections::HashSet, sync::LazyLock};

use crate::{context::Context, evaluator::Evaluator};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Feature<'a, D = fn() -> bool> {
    name: &'a str,
    default_fn: D,
}

impl<'a> Feature<'a> {
    pub const fn new(name: &'a str, default: bool) -> Feature<'a> {
        Feature {
            name,
            default_fn: if default { || true } else { || false },
        }
    }
}

impl<'a, D: Fn() -> bool> Feature<'a, D> {
    pub const fn new_with_default_fn(name: &'a str, default_fn: D) -> Feature<'a, D> {
        Feature { name, default_fn }
    }

    pub const fn name(&self) -> &'a str {
        self.name
    }

    pub fn get_state_in(&self, context: Option<&Context>) -> Option<bool> {
        let context = context.unwrap_or(const { &Context::root() });
        context.evaluator()?.is_enabled(self.name, context)
    }

    #[inline]
    pub fn get_state(&self) -> Option<bool> {
        self.get_state_in(Context::current().as_ref())
    }

    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.is_enabled_in(Context::current().as_ref())
    }

    #[inline]
    pub fn is_enabled_in(&self, context: Option<&Context>) -> bool {
        self.get_state_in(context)
            .unwrap_or_else(|| (self.default_fn)())
    }
}

#[cfg(feature = "feature-registry")]
#[macro_export]
#[doc(hidden)]
macro_rules! __register_feature {
    ($name:literal) => {
        $crate::__reexport::inventory::submit! {
            $crate::feature::RegisteredFeature($name)
        }
    };
}

#[cfg(not(feature = "feature-registry"))]
#[macro_export]
#[doc(hidden)]
macro_rules! __register_feature {
    ($name:literal) => {};
}

#[macro_export]
macro_rules! feature {
    ($name:literal, $default:expr $(,)?) => {{
        $crate::__register_feature!($name);
        $crate::feature::Feature::new_with_default_fn($name, || $default)
    }};

    ($name:literal $(,)?) => {{
        compile_error!("missing default value for feature");
        $crate::feature!($name, false)
    }};
}

#[macro_export]
macro_rules! is_enabled {
    (context: $context:expr, $feature:literal $(, $default:expr)? $(,)?) => {
        $crate::feature!($feature $(, $default)?).is_enabled_in(
            $crate::context::AsContextParam::as_context_param(&$context)
        )
    };

    ($feature:literal $(, $default:expr)? $(,)?) => {
        $crate::feature!($feature $(, $default)?).is_enabled()
    };
}

#[cfg(feature = "feature-registry")]
pub fn known_features() -> &'static HashSet<&'static str> {
    static CACHED: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
        inventory::iter::<RegisteredFeature>()
            .map(|feature| feature.0)
            .collect()
    });
    &CACHED
}

#[cfg(feature = "feature-registry")]
#[doc(hidden)]
pub struct RegisteredFeature(pub &'static str);

#[cfg(feature = "feature-registry")]
inventory::collect!(RegisteredFeature);
