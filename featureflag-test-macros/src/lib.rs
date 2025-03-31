//! Test macros for the `featureflag` crate.
//!
//! This crate shouldn't be used directly, but should be used
//! through its reexports in the `featureflag-test` crate.
#![cfg_attr(docsrs, feature(doc_cfg))]

use quote::ToTokens;

mod utils;
mod with_features;

/// Enable the specified features for use in tests.
///
/// This macro calls `featureflag::evaluator::set_thread_evaluator`, so it
/// should only be used for single-threaded tests.
///
/// Feature values can be any value that implements the `featureflag_test::TestFeature`
/// trait.
///
/// # Examples
///
/// ```no_run
/// #[test]
/// #[with_features("enabled" = true, "disabled" = false)]
/// fn my_test() {
///   assert!(featureflag::is_enabled("enabled"));
///   assert!(!featureflag::is_enabled("disabled"));
/// }
/// ```
#[proc_macro_attribute]
pub fn with_features(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args);
    let input = syn::parse_macro_input!(input);

    with_features::with_features(args, input)
        .map(|output| output.into_token_stream())
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
