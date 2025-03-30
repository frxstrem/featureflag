use std::{collections::HashMap, ops::Deref, sync::RwLock};

use featureflag::{Context, Evaluator, context::ContextRef, fields::Fields};

pub struct TestEvaluator {
    features: RwLock<HashMap<String, Box<dyn TestFeature>>>,
}

impl TestEvaluator {
    pub fn new() -> TestEvaluator {
        TestEvaluator {
            features: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_feature<T: TestFeature>(&self, feature: &str, enabled: T) {
        self.features
            .write()
            .unwrap()
            .insert(feature.to_string(), Box::new(enabled));
    }

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

pub trait TestFeature: Send + Sync + 'static {
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

pub struct TestFields {
    fields: Fields<'static>,
}

impl TestFields {
    pub fn new(fields: Fields<'_>) -> TestFields {
        // very leaky!

        let fields = fields
            .values()
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

pub trait TestContextExt {
    fn test_fields(&self) -> Option<&Fields<'_>>;
}

impl TestContextExt for Context {
    fn test_fields(&self) -> Option<&Fields<'_>> {
        self.extensions()
            .get::<TestFields>()
            .map(|fields| fields.deref())
    }
}
