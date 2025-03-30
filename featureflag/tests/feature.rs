use featureflag::{Feature, evaluator::with_default};
use featureflag_test::TestEvaluator;

#[test]
fn test_is_enabled_macro() {
    let evaluator = TestEvaluator::new();
    evaluator.set_feature("enabled", true);
    evaluator.set_feature("disabled", false);

    with_default(evaluator, || {
        assert!(featureflag::is_enabled!("enabled", false));
        assert!(featureflag::is_enabled!("enabled", true));

        assert!(!featureflag::is_enabled!("disabled", false));
        assert!(!featureflag::is_enabled!("disabled", true));

        assert!(!featureflag::is_enabled!("unknown", false));
        assert!(featureflag::is_enabled!("unknown", true));
    });
}

#[test]
fn test_feature_macro() {
    let evaluator = TestEvaluator::new();
    evaluator.set_feature("enabled", true);
    evaluator.set_feature("disabled", false);

    with_default(evaluator, || {
        const ENABLED_TRUE: Feature = featureflag::feature!("enabled", true);
        const ENABLED_FALSE: Feature = featureflag::feature!("enabled", false);
        const DISABLED_TRUE: Feature = featureflag::feature!("disabled", true);
        const DISABLED_FALSE: Feature = featureflag::feature!("disabled", false);
        const UNKNOWN_TRUE: Feature = featureflag::feature!("unknown", true);
        const UNKNOWN_FALSE: Feature = featureflag::feature!("unknown", false);

        assert_eq!(ENABLED_TRUE.name(), "enabled");
        assert_eq!(ENABLED_FALSE.name(), "enabled");
        assert_eq!(DISABLED_TRUE.name(), "disabled");
        assert_eq!(DISABLED_FALSE.name(), "disabled");
        assert_eq!(UNKNOWN_TRUE.name(), "unknown");
        assert_eq!(UNKNOWN_FALSE.name(), "unknown");

        assert!(ENABLED_TRUE.is_enabled());
        assert!(ENABLED_FALSE.is_enabled());
        assert!(!DISABLED_TRUE.is_enabled());
        assert!(!DISABLED_FALSE.is_enabled());
        assert!(UNKNOWN_TRUE.is_enabled());
        assert!(!UNKNOWN_FALSE.is_enabled());
    });
}
