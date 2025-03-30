use proc_macro_crate::FoundCrate;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
    Expr, ExprLit, Ident, Item, Lit, LitBool, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
};

pub fn with_features(args: TestFeaturesArgs, input: Item) -> syn::Result<impl ToTokens> {
    let Item::Fn(mut input) = input else {
        return Err(syn::Error::new_spanned(
            &input,
            "expected function or method",
        ));
    };

    let evaluator = format_ident!("__evaluator");

    let featureflag = match proc_macro_crate::crate_name("featureflag").unwrap() {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()).into_token_stream(),
    };
    let featureflag_test = match proc_macro_crate::crate_name("featureflag-test").unwrap() {
        FoundCrate::Itself => quote! { crate },
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()).into_token_stream(),
    };

    let features = args
        .test_features
        .into_iter()
        .map(|TestFeatureArg { name, value }| {
            let span = name.span();
            let value = value.unwrap_or_else(|| {
                Expr::Lit(ExprLit {
                    attrs: Vec::new(),
                    lit: Lit::Bool(LitBool::new(true, name.span())),
                })
            });

            quote_spanned! {span=> #evaluator.set_feature(#name, #value); }
        })
        .collect::<Vec<_>>();

    input.block.stmts.insert(
        0,
        parse_quote! {
            {
                let mut #evaluator = #featureflag_test::TestEvaluator::new();
                #( #features )*
                #featureflag::evaluator::set_thread_default(#evaluator);
            };
        },
    );

    Ok(input)
}

pub struct TestFeaturesArgs {
    test_features: Punctuated<TestFeatureArg, Token![,]>,
}

impl Parse for TestFeaturesArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let test_features = Punctuated::parse_terminated(input)?;

        Ok(Self { test_features })
    }
}

pub struct TestFeatureArg {
    name: TestFeatureName,
    value: Option<Expr>,
}

impl Parse for TestFeatureArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<TestFeatureName>()?;

        let value = input
            .parse::<Option<Token![=]>>()?
            .map(|_| input.parse::<Expr>())
            .transpose()?;

        Ok(Self { name, value })
    }
}

pub enum TestFeatureName {
    Ident(Ident),
    LitStr(LitStr),
}

impl ToTokens for TestFeatureName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => {
                let value = ident.to_string();
                let value = value.strip_prefix("r#").unwrap_or(&value);

                let lit_str = LitStr::new(value, ident.span());
                lit_str.to_tokens(tokens);
            }
            Self::LitStr(lit_str) => lit_str.to_tokens(tokens),
        }
    }
}

impl Parse for TestFeatureName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) {
            Ok(Self::Ident(input.parse()?))
        } else if lookahead.peek(LitStr) {
            Ok(Self::LitStr(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::{ToTokens, quote};

    use crate::utils::expand_macro;

    use super::with_features;

    #[test]
    fn test_with_features() {
        let expanded = expand_macro! {
            #[with_features(enabled = true, disabled = false, implicit, custom = custom)]
            #[foo]
            fn test<'a, T: Foo, U, const V: usize>(&mut self, n: i32, Foo(x): Foo) {
                self.beep_boop(n, x)
            }
        };

        let expected = quote! {
            #[foo]
            fn test<'a, T: Foo, U, const V: usize>(&mut self, n: i32, Foo(x): Foo) {
                {
                    let mut __evaluator = featureflag_test::TestEvaluator::new();
                    __evaluator.set_feature("enabled", true);
                    __evaluator.set_feature("disabled", false);
                    __evaluator.set_feature("implicit", true);
                    __evaluator.set_feature("custom", custom);
                    featureflag::evaluator::set_thread_default(__evaluator);
                };

                self.beep_boop(n, x)
            }
        };

        assert_eq!(expanded.to_string(), expected.to_string());
    }
}
