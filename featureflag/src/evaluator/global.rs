use std::{
    borrow::Cow,
    cell::{OnceCell, RefCell},
    fmt,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    sync::OnceLock,
};

use crate::evaluator::{Evaluator, EvaluatorRef};

static GLOBAL_EVALUATOR: OnceLock<EvaluatorRef> = OnceLock::new();

thread_local! {
    static THREAD_EVALUATOR: OnceCell<EvaluatorRef> = const { OnceCell::new() };

    static TASK_EVALUATOR: RefCell<Option<EvaluatorRef>> = const { RefCell::new(None) };
}

pub fn set_global_default<E: Evaluator + Send + Sync + 'static>(evaluator: E) {
    try_set_global_default(evaluator).expect("failed to set global default");
}

pub fn try_set_global_default<E: Evaluator + Send + Sync + 'static>(
    evaluator: E,
) -> Result<(), SetGlobalDefaultError> {
    let mut initialized = false;

    GLOBAL_EVALUATOR.get_or_init(|| {
        initialized = true;
        evaluator.into_ref()
    });

    if initialized {
        Ok(())
    } else {
        Err(SetGlobalDefaultError { _private: () })
    }
}

pub fn set_thread_default<E: Evaluator + Send + Sync + 'static>(evaluator: E) {
    try_set_thread_default(evaluator).expect("failed to set thread default");
}

pub fn try_set_thread_default<E: Evaluator + Send + Sync + 'static>(
    evaluator: E,
) -> Result<(), SetThreadDefaultError> {
    THREAD_EVALUATOR.with(|cell| {
        let mut initialized = false;

        cell.get_or_init(|| {
            initialized = true;
            evaluator.into_ref()
        });

        if initialized {
            Ok(())
        } else {
            Err(SetThreadDefaultError { _private: () })
        }
    })
}

pub fn with_default<E: Evaluator + Send + Sync + 'static, F: FnOnce() -> R, R>(
    evaluator: E,
    f: F,
) -> R {
    evaluator.on_registration();
    with_default_no_registration(evaluator.into_ref(), f)
}

pub(crate) fn with_default_no_registration<F: FnOnce() -> R, R>(
    evaluator: EvaluatorRef,
    f: F,
) -> R {
    let old_thread_evaluator = TASK_EVALUATOR.replace(Some(evaluator));

    let result = catch_unwind(AssertUnwindSafe(f));

    TASK_EVALUATOR.set(old_thread_evaluator);

    match result {
        Ok(result) => result,
        Err(payload) => resume_unwind(payload),
    }
}

pub fn get_default<F: FnOnce(Option<&EvaluatorRef>) -> R, R>(f: F) -> R {
    let evaluator = TASK_EVALUATOR
        .with_borrow(|evaluator| evaluator.clone().map(Cow::Owned))
        .or_else(|| THREAD_EVALUATOR.with(|cell| cell.get().cloned().map(Cow::Owned)))
        .or_else(|| GLOBAL_EVALUATOR.get().map(Cow::Borrowed));

    f(evaluator.as_deref())
}

#[derive(Debug)]
pub struct SetGlobalDefaultError {
    _private: (),
}

impl fmt::Display for SetGlobalDefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("global evaluator already set")
    }
}

impl std::error::Error for SetGlobalDefaultError {}

#[derive(Debug)]
pub struct SetThreadDefaultError {
    _private: (),
}

impl fmt::Display for SetThreadDefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("thread evaluator already set")
    }
}

impl std::error::Error for SetThreadDefaultError {}
