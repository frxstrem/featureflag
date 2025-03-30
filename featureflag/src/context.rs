mod stack;

use std::{fmt, sync::Arc};

use crate::{
    context::stack::GLOBAL_CONTEXT_STACK,
    evaluator::{Evaluator, EvaluatorRef, WeakEvaluatorRef, get_default},
    extensions::Extensions,
    fields::Fields,
};

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
    pub fn new(fields: Fields<'_>) -> Context {
        Context::new_with_parent(Context::current().as_ref(), fields)
    }

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

    pub const fn root() -> Context {
        Context { data: None }
    }

    pub fn is_root(&self) -> bool {
        self.data.is_none()
    }

    pub fn current() -> Option<Context> {
        GLOBAL_CONTEXT_STACK.current()
    }

    pub fn current_or_root() -> Context {
        Context::current().unwrap_or(Context::root())
    }

    pub fn parent(&self) -> Option<&Context> {
        self.data
            .as_ref()?
            .parent
            .as_ref()
            .or(Some(const { &Context::root() }))
    }

    pub fn extensions(&self) -> &Extensions {
        self.data
            .as_ref()
            .map(|data| &data.extensions)
            .unwrap_or(const { &Extensions::new() })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Context> {
        std::iter::successors(Some(self), |context| context.parent())
    }

    pub fn in_scope<F: FnOnce() -> R, R>(&self, f: F) -> R {
        GLOBAL_CONTEXT_STACK.in_scope(self, f)
    }

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

pub struct ContextRef<'a> {
    data: &'a mut Data,
}

impl ContextRef<'_> {
    pub fn parent(&self) -> Option<&Context> {
        self.data.parent.as_ref()
    }

    pub fn extensions(&self) -> &Extensions {
        &self.data.extensions
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.data.extensions
    }

    pub fn iter(&self) -> impl Iterator<Item = &Context> {
        self.data.parent.iter().flat_map(Context::iter)
    }

    pub(crate) fn by_mut(&mut self) -> ContextRef<'_> {
        ContextRef { data: self.data }
    }
}

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
