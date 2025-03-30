use std::{
    cell::RefCell,
    mem,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};

use thread_local::ThreadLocal;

use crate::context::Context;

pub(crate) static GLOBAL_CONTEXT_STACK: ContextStack = ContextStack::new();

pub(crate) struct ContextStack {
    thread_state: ThreadLocal<LocalContextStack>,
}

#[derive(Default)]
struct LocalContextStack {
    current: RefCell<Option<Context>>,
}

impl ContextStack {
    pub const fn new() -> ContextStack {
        ContextStack {
            thread_state: ThreadLocal::new(),
        }
    }

    pub fn in_scope<F: FnOnce() -> R, R>(&self, context: &Context, f: F) -> R {
        let thread_state = self.thread_state.get_or_default();

        let old_context = mem::replace(
            &mut *thread_state.current.borrow_mut(),
            Some(context.clone()),
        );

        let result = catch_unwind(AssertUnwindSafe(f));

        *thread_state.current.borrow_mut() = old_context;

        match result {
            Ok(result) => result,
            Err(payload) => resume_unwind(payload),
        }
    }

    pub fn current(&self) -> Option<Context> {
        let thread_state = self.thread_state.get()?;
        thread_state.current.borrow().clone()
    }
}
