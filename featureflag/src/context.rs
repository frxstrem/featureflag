//! Context values for context-aware features.

mod stack;

use std::{fmt, sync::Arc};

use crate::{
    context::stack::GLOBAL_CONTEXT_STACK,
    evaluator::{Evaluator, EvaluatorRef, WeakEvaluatorRef, get_default},
    extensions::Extensions,
    fields::Fields,
};

/// A context for evaluating feature flags.
///
/// A context contains an [`EvaluatorRef`], a parent context, and a set of custom
/// extensions.
///
/// When a context is created, the evaluator can store custom data in the context
/// based on the context fields.
#[derive(Clone)]
pub struct Context {
    data: Option<Arc<Data>>,
}

struct Data {
    evaluator: WeakEvaluatorRef,
    parent: Option<Context>,
    extensions: Extensions,
}

impl Context {
    /// Creates a new context with the given fields.
    ///
    /// The context is associated with the current evaluator.
    ///
    /// In most cases, you should use the [`context!`] macro to create a context
    /// instead of using this constructor.
    pub fn new(fields: Fields<'_>) -> Context {
        Context::new_with_parent(Context::current().as_ref(), fields)
    }

    /// Creates a new context with the given parent context and fields.
    ///
    /// The context is associated with the current evaluator.
    ///
    /// In most cases, you should use the [`context!`] macro to create a context
    /// instead of using this constructor.
    pub fn new_with_parent(mut parent: Option<&Context>, fields: Fields<'_>) -> Context {
        if parent.is_some_and(|p| p.is_root()) {
            parent = None;
        }

        get_default(|evaluator| {
            let data = match evaluator {
                Some(evaluator) => {
                    let mut data = Data {
                        evaluator: evaluator.downgrade(),
                        parent: parent.cloned(),
                        extensions: Extensions::new(),
                    };

                    evaluator.on_new_context(ContextRef { data: &mut data }, fields);

                    data
                }
                _ => Data {
                    evaluator: WeakEvaluatorRef::new(),
                    parent: parent.cloned(),
                    extensions: Extensions::new(),
                },
            };

            Context {
                data: Some(Arc::new(data)),
            }
        })
    }

    /// Get the root context.
    ///
    /// The root context has no parent and is always associated with the current
    /// evaluator.
    pub const fn root() -> Context {
        Context { data: None }
    }

    /// Check if this context is the root context.
    pub fn is_root(&self) -> bool {
        self.data.is_none()
    }

    /// Get the current context.
    pub fn current() -> Option<Context> {
        GLOBAL_CONTEXT_STACK.current()
    }

    /// Get the current context or the root context if no current context is set.
    pub fn current_or_root() -> Context {
        Context::current().unwrap_or(Context::root())
    }

    /// Get the parent context of this context.
    ///
    /// All contexts except the root context have a parent context, so this only
    /// returns `None` for the root context.
    pub fn parent(&self) -> Option<&Context> {
        self.data
            .as_ref()?
            .parent
            .as_ref()
            .or(Some(const { &Context::root() }))
    }

    /// Get a read-only reference to the extensions of this context.
    pub fn extensions(&self) -> &Extensions {
        self.data
            .as_ref()
            .map(|data| &data.extensions)
            .unwrap_or(const { &Extensions::new() })
    }

    /// Iterate over this context and its parents.
    pub fn iter(&self) -> impl Iterator<Item = &Context> {
        std::iter::successors(Some(self), |context| context.parent())
    }

    /// Run a function with this context as the current context.
    pub fn in_scope<F: FnOnce() -> R, R>(&self, f: F) -> R {
        GLOBAL_CONTEXT_STACK.in_scope(self, f)
    }

    /// Get the evaluator associated with this context.
    pub(crate) fn evaluator(&self) -> Option<EvaluatorRef> {
        match &self.data {
            Some(data) => data.evaluator.upgrade(),
            None => {
                // root context always uses the current default evaluator
                get_default(|evaluator| evaluator.cloned())
            }
        }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context").finish_non_exhaustive()
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        if let Some(evaluator) = self.evaluator.upgrade() {
            evaluator.on_close_context(ContextRef { data: self })
        }
    }
}

/// A mutable reference to a context being created or destroyed.
pub struct ContextRef<'a> {
    data: &'a mut Data,
}

impl ContextRef<'_> {
    /// Get the parent context of this context.
    ///
    /// See [`Context::parent`] for more details.
    pub fn parent(&self) -> Option<&Context> {
        self.data.parent.as_ref()
    }

    /// Get a read-only reference to the extensions of this context.
    pub fn extensions(&self) -> &Extensions {
        &self.data.extensions
    }

    /// Get a mutable reference to the extensions of this context.
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.data.extensions
    }

    /// Recursively iterate over this context's parents.
    ///
    /// Because the `ContextRef` is used before the context is created, and
    /// after it is destroyed, the iterator will not include the context itself.
    pub fn iter(&self) -> impl Iterator<Item = &Context> {
        self.data.parent.iter().flat_map(Context::iter)
    }

    pub(crate) fn by_mut(&mut self) -> ContextRef<'_> {
        ContextRef { data: self.data }
    }
}

/// Create a new context with the given fields.
///
/// The fields are specified as a comma-separated list of `key = value` pairs.
/// Field values can be any type that implements the [`ToValue`](crate::value::ToValue) trait.
///
/// A parent context can be specified with `parent: <parent>`.
///
/// # Examples
///
/// ```
/// let a = context!(foo = 1, bar = "baz");
/// let b = context!(parent: a, foo = 2);
/// let c = context!(parent: None, foo = 3);
/// ```
#[macro_export]
macro_rules! context {
    (parent: $parent:expr $(, $($fields:tt)*)?) => {
        $crate::context::Context::new_with_parent(
            $crate::context::AsContextParam::as_context_param(
                &$parent
            ),
            $crate::fields!($($($fields)*)?),
        )
    };
    ($($fields:tt)*) => {
        $crate::context::Context::new($crate::fields!($($fields)*))
    };
}

// Allow references from doc comments before the macro definition.
#[allow(unused_imports)]
use crate::context;

/// Helper trait for macros to accept different types as context parameters.
#[doc(hidden)]
pub trait AsContextParam {
    fn as_context_param(&self) -> Option<&Context>;
}

impl AsContextParam for Context {
    fn as_context_param(&self) -> Option<&Context> {
        Some(self)
    }
}

impl AsContextParam for Option<Context> {
    fn as_context_param(&self) -> Option<&Context> {
        self.as_ref()
    }
}

impl AsContextParam for () {
    fn as_context_param(&self) -> Option<&Context> {
        None
    }
}
