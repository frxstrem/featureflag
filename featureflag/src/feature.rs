//! Feature flags.

#[cfg(feature = "feature-registry")]
use std::{collections::HashSet, sync::LazyLock};

use crate::{context::Context, evaluator::Evaluator};

/// Feature flag definition.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Feature<'a, D = fn() -> bool> {
    name: &'a str,
    default_fn: D,
}

impl<'a> Feature<'a> {
    /// Create a new feature flag.
    ///
    /// The default value is used when the evaluator returns `None` for the feature.
    ///
    /// In most cases, you should use the [`feature!`] macro instead of this
    /// constructor.
    pub const fn new(name: &'a str, default: bool) -> Feature<'a> {
        Feature {
            name,
            default_fn: if default { || true } else { || false },
        }
    }
}

impl<'a, D: Fn() -> bool> Feature<'a, D> {
    /// Create a new feature flag with a custom default function.
    ///
    /// The default function is called when the evaluator returns `None` for the feature.
    ///
    /// In most cases, you should use the [`feature!`] macro instead of this
    /// constructor.
    pub const fn new_with_default_fn(name: &'a str, default_fn: D) -> Feature<'a, D> {
        Feature { name, default_fn }
    }

    /// Get the name of the feature.
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Get the state of the feature in the given context.
    pub fn get_state_in(&self, context: Option<&Context>) -> Option<bool> {
        let context = context.unwrap_or(const { &Context::root() });
        context.evaluator()?.is_enabled(self.name, context)
    }

    /// Get the state of the feature in the current context.
    #[inline]
    pub fn get_state(&self) -> Option<bool> {
        self.get_state_in(Context::current().as_ref())
    }

    /// Check if the feature is enabled in the current context.
    ///
    /// If the current evaluator returns `None` for the feature, the default
    /// of this feature is used.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.is_enabled_in(Context::current().as_ref())
    }

    /// Check if the feature is enabled in the given context.
    ///
    /// If the context's evaluator returns `None` for the feature, the default
    /// of this feature is used.
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

/// Define a feature flag at compile-time.
///
/// The macro takes two arguments: the name of the feature, and an optional default
/// value. The default value argument is evaluated each time the feature is using
/// its default value.
///
/// If the `feature-registry` feature is enabled, the feature will be registered
/// globally and can be accessed using the [`known_features`] function.
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

/// Check if a feature is enabled.
///
/// `is_enabled!("feature", default)` is equivalent to `feature!("feature", default).is_enabled()`.
///
/// A context can be passed to use instead of the current context, by passing
/// `is_enabled!(context: some_context, "feature", default)`.
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

// Allow references from doc comments before the macro definition.
#[allow(unused_imports)]
use crate::{feature, is_enabled};

#[cfg(feature = "feature-registry")]
#[cfg_attr(docsrs, doc(cfg(feature = "feature-registry")))]
/// Get all feature flags registered with [`feature!`] or [`is_enabled!`].
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
