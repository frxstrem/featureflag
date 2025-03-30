use quote::ToTokens;

mod utils;
mod with_features;

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
