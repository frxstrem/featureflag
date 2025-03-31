//! Evaluators of feature flags.
//!
//! This module defines the [`Evaluator`] trait, which is used to evaluate feature flags
//! at runtime. It also provides utilities for composing evaluators, such as
//! [`Filter`] and [`Chain`], as well as a default evaluator, [`NoEvaluator`], which
//! always returns `None` for feature flags.
//!
//! # Global evaluator
//!
//! The global evaluator is used by default evaluating feature flags. It can be
//! set globally using the [`set_global_default`] and [`try_set_global_default`] functions,
//! locally to a thread using the [`set_thread_default`] and [`try_set_thread_default`] functions,
//! or in a specific scope using the [`with_default`] or [`AnyExt::wrap_evaluator`](crate::utils::AnyExt::wrap_evaluator)
//! functions. The global evaluator can be accessed using the [`get_default`] function.

mod global;

use std::sync::{Arc, LazyLock, Weak};

use crate::{
    context::{Context, ContextRef},
    fields::Fields,
};

pub use self::global::*;

/// Evaluator of feature flags.
///
/// This trait is used to evaluate feature flags at runtime. It provides methods
/// to check if a feature is enabled, and to handle registration and context
/// management.
pub trait Evaluator: Send + Sync {
    /// Checks if a feature is enabled in the given context.
    ///
    /// # Returns
    ///
    /// - `Some(true)` if the feature is explicitly enabled.
    /// - `Some(false)` if the feature is explicitly disabled.
    /// - `None` if the feature's default value should be used.
    fn is_enabled(&self, feature: &str, context: &Context) -> Option<bool>;

    /// Called when the evaluator is registered.
    ///
    /// Functions like [`set_global_default`], [`set_thread_default`] and [`with_default`]
    /// will call this method to notify the evaluator that it has been registered. The evaluator
    /// can use this method to perform any necessary initialization.
    ///
    /// This method may be called multiple times, so the evaluator should ensure
    /// that it can handle this.
    fn on_registration(&self) {}

    /// Called when a new context is created.
    ///
    /// The evaluator can use this method to store any context-specific data.
    /// Fields are not stored in the context, so the evaluator should store them
    /// if they are needed to evaluate feature flags.
    fn on_new_context(&self, context: ContextRef<'_>, fields: Fields<'_>) {
        let _ = (context, fields);
    }

    /// Called when a context is closed.
    ///
    /// If a context has been closed, this is called after the last reference
    /// to it has been dropped. The evaluator can use this method to clean up
    /// any context-specific data, if needed.
    fn on_close_context(&self, context: ContextRef<'_>) {
        let _ = context;
    }

    /// Converts the evaluator into an [`EvaluatorRef`].
    ///
    /// The default implementation calls `EvaluatorRef::from_arc(Arc::new(self))`.
    ///
    /// For most types, the default implementation should not be overriden. It
    /// should only be overriden if it can be converted into an [`EvaluatorRef`]
    /// more efficiently than the default implementation.
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

/// Evaluator that always returns `None` for all features.
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

/// A shared reference to an [`Evaluator`].
#[derive(Clone)]
pub struct EvaluatorRef {
    arc: Arc<dyn Evaluator + Send + Sync>,
}

impl EvaluatorRef {
    /// Creates a new [`EvaluatorRef`] from an [`Arc<dyn Evaluator>`].
    pub fn from_arc(arc: Arc<dyn Evaluator + Send + Sync>) -> Self {
        Self { arc }
    }

    /// Downgrade into a [`WeakEvaluatorRef`].
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

/// A weak reference to an [`Evaluator`].
#[derive(Clone)]
pub struct WeakEvaluatorRef {
    weak: Weak<dyn Evaluator + Send + Sync>,
}

impl WeakEvaluatorRef {
    /// Creates a detached new [`WeakEvaluatorRef`].
    ///
    /// Calling [`upgrade`](Self::upgrade) on this reference will always return `None`.
    pub const fn new() -> WeakEvaluatorRef {
        Self {
            weak: Weak::<NoEvaluator>::new(),
        }
    }

    /// Attempt to upgrade the weak reference to a strong reference.
    pub fn upgrade(&self) -> Option<EvaluatorRef> {
        self.weak.upgrade().map(|arc| EvaluatorRef { arc })
    }
}

impl Default for WeakEvaluatorRef {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for [`Evaluator`].
pub trait EvaluatorExt: Evaluator {
    /// Filter features based on a filter function.
    ///
    /// This method will only call the evaluator if the filter function returns `true`,
    /// and will always return `None` for other features.
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

    /// Chain two evaluators together.
    ///
    /// This method will call the first evaluator, and if it returns `None`, it will
    /// call the second evaluator. If both evaluators return `None`, the result will be `None`.
    fn chain<U>(self, other: U) -> Chain<Self, U>
    where
        Self: Sized,
        U: Evaluator,
    {
        Chain(self, other)
    }
}

impl<E: ?Sized + Evaluator> EvaluatorExt for E {}

/// Filter evaluator, see [`EvaluatorExt::filter`].
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

/// Chain evaluator, see [`EvaluatorExt::chain`].
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
