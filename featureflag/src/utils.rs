//! This module provides utilities for wrapping types with a [`Context`] or [`Evaluator`].

use std::{pin::Pin, task::Poll};

use crate::{
    Context, Evaluator,
    evaluator::{EvaluatorRef, NoEvaluator, get_default, with_default_no_registration},
};

/// Extension trait for all types.
pub trait AnyExt {
    /// Wraps the given value with a [`Context`].
    ///
    /// If `Self` is a future, then `WrapContext<Self>` is also a future that
    /// will be run within the given context.
    ///
    /// If `Self` is a stream and the `futures` feature is enabled, then
    /// `WrapContext<Self>` is also a stream that will be run within the
    /// given context.
    fn wrap_context(self, context: Context) -> WrapContext<Self>
    where
        Self: Sized,
    {
        WrapContext {
            context,
            inner: self,
        }
    }

    /// Wraps the given value with the current [`Context`].
    ///
    /// See [`AnyExt::wrap_context`] and [`Context::current`] for more details.
    fn inherit_context(self) -> WrapContext<Self>
    where
        Self: Sized,
    {
        self.wrap_context(Context::current_or_root())
    }

    /// Wraps the given value with an [`Evaluator`].
    ///
    /// If `Self` is a future, then `WrapEvaluator<Self>` is also a future that
    /// will be run within the given evaluator, as if called within
    /// [`with_default`](crate::evaluator::with_default).
    ///
    /// If `Self` is a stream and the `futures` feature is enabled, then
    /// `WrapEvaluator<Self>` is also a stream that will be run within the
    /// given evaluator, as if called within
    /// [`with_default`](crate::evaluator::with_default).
    fn wrap_evaluator(self, evaluator: EvaluatorRef) -> WrapEvaluator<Self>
    where
        Self: Sized,
    {
        WrapEvaluator {
            evaluator,
            registered: false,
            inner: self,
        }
    }

    /// Wraps the given value with the current [`Evaluator`].
    ///
    /// See [`AnyExt::wrap_evaluator`] and [`get_default`] for more details.
    fn inherit_evaluator(self) -> WrapEvaluator<Self>
    where
        Self: Sized,
    {
        let evaluator =
            get_default(|evaluator| evaluator.cloned()).unwrap_or_else(|| NoEvaluator.into_ref());
        WrapEvaluator {
            evaluator,
            registered: true,
            inner: self,
        }
    }
}

/// Wraps a type with a [`Context`].
///
/// See [`AnyExt::wrap_context`] for more details.
pub struct WrapContext<T: ?Sized> {
    context: Context,
    inner: T,
}

impl<Fut: ?Sized + Future> Future for WrapContext<Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Fut::Output> {
        let (context, inner) = unsafe {
            let this = self.get_unchecked_mut();
            (&this.context, Pin::new_unchecked(&mut this.inner))
        };

        context.in_scope(|| inner.poll(cx))
    }
}

#[cfg(feature = "futures")]
impl<S: ?Sized + futures_core::Stream> futures_core::Stream for WrapContext<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<S::Item>> {
        let (context, inner) = unsafe {
            let this = self.get_unchecked_mut();
            (&this.context, Pin::new_unchecked(&mut this.inner))
        };

        context.in_scope(|| inner.poll_next(cx))
    }
}

/// Wraps a type with an [`Evaluator`].
///
/// See [`AnyExt::wrap_evaluator`] for more details.
pub struct WrapEvaluator<T: ?Sized> {
    evaluator: EvaluatorRef,
    registered: bool,
    inner: T,
}

impl<Fut: ?Sized + Future> Future for WrapEvaluator<Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Fut::Output> {
        let (evaluator, registered, inner) = unsafe {
            let this = self.get_unchecked_mut();
            (
                &this.evaluator,
                &mut this.registered,
                Pin::new_unchecked(&mut this.inner),
            )
        };

        if !*registered {
            evaluator.on_registration();
            *registered = true;
        }

        with_default_no_registration(evaluator.clone(), || inner.poll(cx))
    }
}

#[cfg(feature = "futures")]
impl<S: ?Sized + futures_core::Stream> futures_core::Stream for WrapEvaluator<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<S::Item>> {
        let (evaluator, registered, inner) = unsafe {
            let this = self.get_unchecked_mut();
            (
                &this.evaluator,
                &mut this.registered,
                Pin::new_unchecked(&mut this.inner),
            )
        };

        if !*registered {
            evaluator.on_registration();
            *registered = true;
        }

        with_default_no_registration(evaluator.clone(), || inner.poll_next(cx))
    }
}
