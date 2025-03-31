//! Test utilities for the [`featureflag`] crate.
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::{collections::HashMap, ops::Deref, sync::RwLock};

use featureflag::{Context, Evaluator, context::ContextRef, fields::Fields};

pub use featureflag_test_macros::*;

/// A test evaluator that allows setting features for testing purposes.
pub struct TestEvaluator {
    features: RwLock<HashMap<String, Box<dyn TestFeature>>>,
}

impl TestEvaluator {
    /// Create a new `TestEvaluator`.
    pub fn new() -> TestEvaluator {
        TestEvaluator {
            features: RwLock::new(HashMap::new()),
        }
    }

    /// Set the state of a feature.
    ///
    /// The feature can be set to any value that implements `TestFeature`, which
    /// allows for complex logic to determine if a feature is enabled. `TestFeature`
    /// is automatically implemented for `bool`, `Option<bool>` and
    /// `Fn(&Context) -> impl TestFeature`.
    pub fn set_feature<T: TestFeature>(&self, feature: &str, enabled: T) {
        self.features
            .write()
            .unwrap()
            .insert(feature.to_string(), Box::new(enabled));
    }

    /// Unset a feature.
    pub fn clear_feature(&self, feature: &str) {
        self.features.write().unwrap().remove(feature);
    }
}

impl Default for TestEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for TestEvaluator {
    fn is_enabled(&self, feature: &str, _context: &crate::Context) -> Option<bool> {
        self.features
            .read()
            .unwrap()
            .get(feature)
            .and_then(|f| f.is_enabled(_context))
    }

    fn on_new_context(&self, mut context: ContextRef<'_>, fields: Fields<'_>) {
        let fields = TestFields::new(fields);
        context.extensions_mut().insert(fields);
    }
}

/// A trait for types that can determine if a feature is enabled.
pub trait TestFeature: Send + Sync + 'static {
    /// Check if the feature is enabled.
    fn is_enabled(&self, context: &Context) -> Option<bool>;
}

impl<T: TestFeature> TestFeature for Option<T> {
    fn is_enabled(&self, context: &Context) -> Option<bool> {
        self.as_ref()?.is_enabled(context)
    }
}

impl TestFeature for bool {
    fn is_enabled(&self, _context: &Context) -> Option<bool> {
        Some(*self)
    }
}

impl<F, O> TestFeature for F
where
    F: Fn(&Context) -> O + Send + Sync + 'static,
    O: TestFeature,
{
    fn is_enabled(&self, context: &Context) -> Option<bool> {
        self(context).is_enabled(context)
    }
}

/// Extension type for [`Context`] that allows access to fields set on a context
/// when using the [`TestEvaluator`].
///
/// This type is not intended to be used directly. Instead, use [`TestContextExt::test_fields`]
/// to access the fields.
struct TestFields {
    fields: Fields<'static>,
}

impl TestFields {
    fn new(fields: Fields<'_>) -> TestFields {
        // very leaky!

        let fields = fields
            .pairs()
            .map(|(k, v)| (&*k.to_string().leak(), v.to_static()))
            .collect::<Vec<_>>()
            .leak();

        TestFields {
            fields: Fields::new(fields),
        }
    }
}

impl Deref for TestFields {
    type Target = Fields<'static>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

/// Extension trait for [`Context`] that provides access to the fields set on
/// the context when using the [`TestEvaluator`].
pub trait TestContextExt {
    /// Get the fields set on the context.
    ///
    /// This method will only work with contexts that have been created when
    /// using [`TestEvaluator`].
    fn test_fields(&self) -> Option<&Fields<'_>>;
}

impl TestContextExt for Context {
    fn test_fields(&self) -> Option<&Fields<'_>> {
        self.extensions()
            .get::<TestFields>()
            .map(|fields| fields.deref())
    }
}
