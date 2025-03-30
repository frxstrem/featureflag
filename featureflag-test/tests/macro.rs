#![allow(clippy::needless_lifetimes, clippy::extra_unused_lifetimes, dead_code)]

use std::marker::PhantomData;

use featureflag::{Context, context};
use featureflag_test::{TestContextExt, with_features};

#[test]
#[with_features(enabled = true, disabled = false, custom = custom, implicit)]
fn test_macro() {
    assert!(featureflag::is_enabled!("enabled", false));
    assert!(featureflag::is_enabled!("enabled", true));

    assert!(!featureflag::is_enabled!("disabled", false));
    assert!(!featureflag::is_enabled!("disabled", true));

    assert!(featureflag::is_enabled!("implicit", false));
    assert!(featureflag::is_enabled!("implicit", true));

    assert!(!featureflag::is_enabled!("unknown", false));
    assert!(featureflag::is_enabled!("unknown", true));

    assert!(!featureflag::is_enabled!("custom", false));
    assert!(featureflag::is_enabled!("custom", true));

    context!(foo = true).in_scope(|| {
        assert!(featureflag::is_enabled!("custom", false));
        assert!(featureflag::is_enabled!("custom", true));

        context!(foo = false).in_scope(|| {
            assert!(!featureflag::is_enabled!("custom", false));
            assert!(!featureflag::is_enabled!("custom", true));
        });
    });
}

#[test]
#[with_features("enabled" = true, "disabled" = false, "custom" = custom, "implicit")]
fn test_macro_litstr() {
    assert!(featureflag::is_enabled!("enabled", false));
    assert!(featureflag::is_enabled!("enabled", true));

    assert!(!featureflag::is_enabled!("disabled", false));
    assert!(!featureflag::is_enabled!("disabled", true));

    assert!(featureflag::is_enabled!("implicit", false));
    assert!(featureflag::is_enabled!("implicit", true));

    assert!(!featureflag::is_enabled!("unknown", false));
    assert!(featureflag::is_enabled!("unknown", true));

    assert!(!featureflag::is_enabled!("custom", false));
    assert!(featureflag::is_enabled!("custom", true));

    context!(foo = true).in_scope(|| {
        assert!(featureflag::is_enabled!("custom", false));
        assert!(featureflag::is_enabled!("custom", true));

        context!(foo = false).in_scope(|| {
            assert!(!featureflag::is_enabled!("custom", false));
            assert!(!featureflag::is_enabled!("custom", true));
        });
    });
}

fn custom(context: &Context) -> Option<bool> {
    context
        .iter()
        .filter_map(|c| c.test_fields())
        .filter_map(|f| f.get("foo"))
        .filter_map(|v| v.as_bool())
        .next()
}

// check that the macro compiles inside an impl block
struct Foo<'a, T, const N: usize> {
    _phantom: PhantomData<&'a T>,
}
fn foo<'a, 'b, T, U, const N: usize, const M: usize>() {}
impl<'a, T, const N: usize> Foo<'a, T, N> {
    #[with_features("enabled" = true, "disabled" = false, "custom" = custom, "implicit")]
    fn test_macro_in_impl<'b, U, const M: usize>() {
        foo::<T, U, N, M>();
    }
}
