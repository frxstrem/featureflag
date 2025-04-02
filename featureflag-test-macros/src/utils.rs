#[cfg(test)]
macro_rules! expand_macro {
    ( #[$name:ident $(($($args:tt)*))? ] $($tokens:tt)* ) => {
        $name( syn::parse_quote! { $($($args)*)? }, syn::parse_quote! { $($tokens)* } )
            .map(|output| output.into_token_stream())
            .unwrap_or_else(|err| err.into_compile_error())
    };
}

#[cfg(test)]
pub(crate) use expand_macro;
use proc_macro_crate::FoundCrate;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// Wrapper for `proc_macro_crate::crate_name` that handles the case where the
/// crate is not found in the current package without failing.
///
/// This is necessary for the tests of this package, as they cannot have other
/// `featureflag` in `dev-dependencies`.
pub(crate) fn crate_name(orig_name: &str) -> TokenStream {
    match proc_macro_crate::crate_name(orig_name) {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let name = Ident::new(&name, Span::call_site());
            quote! { ::#name }
        }
        Err(proc_macro_crate::Error::CrateNotFound { crate_name, .. }) => {
            let crate_name = crate_name.replace('-', "_");

            let name = Ident::new(&crate_name, Span::call_site());
            quote! { ::#name }
        }
        Err(err) => panic!("{err}"),
    }
}
