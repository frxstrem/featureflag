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
