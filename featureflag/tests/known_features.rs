use std::collections::HashSet;

use featureflag::{Feature, feature::known_features};

#[allow(dead_code)]
fn func() {
    featureflag::feature!("a", false);
    featureflag::feature!("b", true);
    featureflag::is_enabled!("c", false);
    featureflag::is_enabled!("d", true);

    Feature::new("dynamic1", false).is_enabled();
}

#[test]
fn test_known_features() {
    Feature::new("dynamic2", false).is_enabled();

    // these are all of the features that are used in the same program
    let expected = [
        "a", "b", "c", "d", /* not expected: "dynamic1", "dynamic2" */
    ]
    .into_iter()
    .collect::<HashSet<_>>();

    assert_eq!(known_features(), &expected);
}
