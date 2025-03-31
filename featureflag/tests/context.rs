#![allow(missing_docs)]

use featureflag::{Context, context, evaluator::with_default};
use featureflag_test::{TestContextExt, TestEvaluator};

#[test]
fn test_context() {
    let evaluator = TestEvaluator::new();
    evaluator.set_feature("foo", |c: &Context| {
        c.iter()
            .find_map(|c| c.test_fields().and_then(|f| f.get("foo")))
            .and_then(|v| v.as_bool())
    });

    with_default(evaluator, || {
        assert!(!featureflag::is_enabled!("foo", false));

        context!().in_scope(|| {
            assert!(!featureflag::is_enabled!("foo", false));

            context!(foo = true).in_scope(|| {
                assert!(featureflag::is_enabled!("foo", false));
            });
        });

        context!(foo = true).in_scope(|| {
            assert!(featureflag::is_enabled!("foo", false));

            context!(bar = false).in_scope(|| {
                assert!(featureflag::is_enabled!("foo", false));
            });

            context!(foo = false).in_scope(|| {
                assert!(!featureflag::is_enabled!("foo", false));
            });
        });
    });
}
