//! Feature flagging facade for Rust.
//!
//! This library provides a flexible and extensible way to control feature flags
//! in Rust applications.
//!
//! The core trait of this library is the [`Evaluator`], which is used to evaluate
//! feature flags at runtime.
//!
//! The [`context!`] macro and [`Context`] type can be in evaluators to provide
//! contextual information for feature flag evaluation, such as user ID and session ID.
//!
//! The [`is_enabled!`] macro is the primary way to check if a feature is enabled.
//! This macro takes a feature name and a default value, and returns a boolean
//! indicating whether the feature is enabled or not. Alternatively, the [`feature!`]
//! macro can be used to store a [`Feature`] is a variable or constant, or the
//! [`Feature::new`] or [`Feature::new_with_default_fn`] methods can be used
//! directly to create new feature flags at runtime.
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod context;
pub mod evaluator;
pub mod extensions;
pub mod feature;
pub mod fields;
pub mod utils;
pub mod value;

pub use crate::{
    context::Context,
    evaluator::{Evaluator, set_global_default, try_set_global_default},
    feature::Feature,
};

#[doc(hidden)]
pub mod __reexport {

    #[cfg(feature = "feature-registry")]
    pub use inventory;
}
