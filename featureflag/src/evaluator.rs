mod global;

use std::sync::{Arc, LazyLock, Weak};

use crate::{
    context::{Context, ContextRef},
    fields::Fields,
};

pub use self::global::*;

pub trait Evaluator: Send + Sync {
    fn is_enabled(&self, feature: &str, context: &Context) -> Option<bool>;

    fn on_registration(&self) {}

    fn on_new_context(&self, context: ContextRef<'_>, fields: Fields<'_>) {
        let _ = (context, fields);
    }

    fn on_close_context(&self, context: ContextRef<'_>) {
        let _ = context;
    }

    fn into_ref(self) -> EvaluatorRef
    where
        Self: Sized + 'static,
    {
        EvaluatorRef::from_arc(Arc::new(self))
    }
}

impl<E: Evaluator> Evaluator for Arc<E> {
    fn is_enabled(&self, feature: &str, context: &Context) -> Option<bool> {
        self.as_ref().is_enabled(feature, context)
    }

    fn on_registration(&self) {
        self.as_ref().on_registration()
    }

    fn on_new_context(&self, context: ContextRef<'_>, fields: Fields<'_>) {
        self.as_ref().on_new_context(context, fields)
    }

    fn on_close_context(&self, context: ContextRef<'_>) {
        self.as_ref().on_close_context(context)
    }

    fn into_ref(self) -> EvaluatorRef
    where
        Self: Sized + 'static,
    {
        EvaluatorRef::from_arc(self)
    }
}

impl Evaluator for Arc<dyn Evaluator + Send + Sync> {
    fn is_enabled(&self, feature: &str, context: &Context) -> Option<bool> {
        self.as_ref().is_enabled(feature, context)
    }

    fn on_registration(&self) {
        self.as_ref().on_registration()
    }

    fn on_new_context(&self, context: ContextRef<'_>, fields: Fields<'_>) {
        self.as_ref().on_new_context(context, fields)
    }

    fn on_close_context(&self, context: ContextRef<'_>) {
        self.as_ref().on_close_context(context)
    }

    fn into_ref(self) -> EvaluatorRef
    where
        Self: Sized + Send + Sync + 'static,
    {
        EvaluatorRef::from_arc(self)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct NoEvaluator;

impl Evaluator for NoEvaluator {
    fn is_enabled(&self, _feature: &str, _context: &Context) -> Option<bool> {
        None
    }

    fn into_ref(self) -> EvaluatorRef {
        static GLOBAL_NO_EVALUATOR: LazyLock<Arc<NoEvaluator>> =
            LazyLock::new(|| Arc::new(NoEvaluator));
        EvaluatorRef::from_arc(GLOBAL_NO_EVALUATOR.clone())
    }
}

#[derive(Clone)]
pub struct EvaluatorRef {
    arc: Arc<dyn Evaluator + Send + Sync>,
}

impl EvaluatorRef {
    pub fn from_arc(arc: Arc<dyn Evaluator + Send + Sync>) -> Self {
        Self { arc }
    }

    pub fn downgrade(&self) -> WeakEvaluatorRef {
        WeakEvaluatorRef {
            weak: Arc::downgrade(&self.arc),
        }
    }
}

impl Evaluator for EvaluatorRef {
    fn is_enabled(&self, feature: &str, context: &Context) -> Option<bool> {
        self.arc.is_enabled(feature, context)
    }

    fn on_registration(&self) {
        self.arc.on_registration()
    }

    fn on_new_context(&self, context: ContextRef<'_>, fields: Fields<'_>) {
        self.arc.on_new_context(context, fields)
    }

    fn on_close_context(&self, context: ContextRef<'_>) {
        self.arc.on_close_context(context)
    }

    fn into_ref(self) -> EvaluatorRef {
        self
    }
}

#[derive(Clone)]
pub struct WeakEvaluatorRef {
    weak: Weak<dyn Evaluator + Send + Sync>,
}

impl WeakEvaluatorRef {
    pub const fn new() -> WeakEvaluatorRef {
        Self {
            weak: Weak::<NoEvaluator>::new(),
        }
    }

    pub fn upgrade(&self) -> Option<EvaluatorRef> {
        self.weak.upgrade().map(|arc| EvaluatorRef { arc })
    }
}

impl Default for WeakEvaluatorRef {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EvaluatorExt: Evaluator {
    fn filter<F>(self, filter_fn: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        Filter {
            evaluator: self,
            filter_fn,
        }
    }

    fn chain<U>(self, other: U) -> Chain<Self, U>
    where
        Self: Sized,
        U: Evaluator,
    {
        Chain(self, other)
    }
}

impl<E: ?Sized + Evaluator> EvaluatorExt for E {}

pub struct Filter<E, F> {
    filter_fn: F,
    evaluator: E,
}

impl<E, F> Evaluator for Filter<E, F>
where
    E: Evaluator,
    F: Fn(&str) -> bool + Send + Sync + 'static,
{
    fn is_enabled(&self, feature: &str, context: &Context) -> Option<bool> {
        if (self.filter_fn)(feature) {
            self.evaluator.is_enabled(feature, context)
        } else {
            None
        }
    }

    fn on_registration(&self) {
        self.evaluator.on_registration()
    }

    fn on_new_context(&self, context: ContextRef<'_>, fields: Fields<'_>) {
        self.evaluator.on_new_context(context, fields)
    }

    fn on_close_context(&self, context: ContextRef<'_>) {
        self.evaluator.on_close_context(context)
    }
}

pub struct Chain<T, U>(T, U);

impl<T: Evaluator, U: Evaluator> Evaluator for Chain<T, U> {
    fn is_enabled(&self, feature: &str, context: &Context) -> Option<bool> {
        self.0
            .is_enabled(feature, context)
            .or_else(|| self.1.is_enabled(feature, context))
    }

    fn on_registration(&self) {
        self.0.on_registration();
        self.1.on_registration();
    }

    fn on_new_context(&self, mut context: ContextRef<'_>, fields: Fields<'_>) {
        self.0.on_new_context(context.by_mut(), fields.clone());
        self.1.on_new_context(context, fields);
    }

    fn on_close_context(&self, mut context: ContextRef<'_>) {
        self.0.on_close_context(context.by_mut());
        self.1.on_close_context(context);
    }

    fn into_ref(self) -> EvaluatorRef
    where
        Self: Sized + 'static,
    {
        EvaluatorRef::from_arc(Arc::new(self))
    }
}
