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

/// Set the global evaluator.
///
/// # Panics
///
/// Panics if the global evaluator is already set.
/// For a non-panicking version, use [`try_set_global_default`].
pub fn set_global_default<E: Evaluator + Send + Sync + 'static>(evaluator: E) {
    try_set_global_default(evaluator).expect("failed to set global default");
}

/// Set the global evaluator.
///
/// # Errors
///
/// Returns an error if the global evaluator is already set.
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

/// Set the thread evaluator.
///
/// This function overrides the global evaluator set by [`set_global_default`].
///
/// # Panics
///
/// Panics if the thread evaluator is already set.
/// For a non-panicking version, use [`try_set_thread_default`].
pub fn set_thread_default<E: Evaluator + Send + Sync + 'static>(evaluator: E) {
    try_set_thread_default(evaluator).expect("failed to set thread default");
}

/// Set the thread evaluator.
///
/// This function overrides the global evaluator set by [`set_global_default`].
///
/// # Errors
///
/// Returns an error if the thread evaluator is already set.
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

/// Set the evaluator inside the given closure.
///
/// This function overrides the thread evaluator set by [`set_global_default`]
/// and [`set_thread_default`].
pub fn with_default<E: Evaluator + Send + Sync + 'static, F: FnOnce() -> R, R>(
    evaluator: E,
    f: F,
) -> R {
    evaluator.on_registration();
    with_default_no_registration(evaluator.into_ref(), f)
}

/// Set the evaluator inside the given closure, without calling
/// [`Evaluator::on_registration`].
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

/// Get the default evaluator currently in scope.
///
/// This function will use the first of the following:
/// 1. The evaluator set by [`with_default`].
/// 2. The evaluator set by [`set_thread_default`].
/// 3. The evaluator set by [`set_global_default`].
pub fn get_default<F: FnOnce(Option<&EvaluatorRef>) -> R, R>(f: F) -> R {
    let evaluator = TASK_EVALUATOR
        .with_borrow(|evaluator| evaluator.clone().map(Cow::Owned))
        .or_else(|| THREAD_EVALUATOR.with(|cell| cell.get().cloned().map(Cow::Owned)))
        .or_else(|| GLOBAL_EVALUATOR.get().map(Cow::Borrowed));

    f(evaluator.as_deref())
}

/// Error returned when trying to set the global evaluator
/// when one is already set.
///
/// This error is returned by [`try_set_global_default`].
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

/// Error returned when trying to set the thread evaluator
/// when one is already set.
///
/// This error is returned by [`try_set_thread_default`].
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
