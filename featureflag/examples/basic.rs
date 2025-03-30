#![allow(missing_docs)]

use featureflag::{
    Context, Evaluator, context, context::ContextRef, fields::Fields, set_global_default,
};

struct SomeEvaluator;

impl SomeEvaluator {
    fn new() -> Self {
        SomeEvaluator
    }
}

impl Evaluator for SomeEvaluator {
    fn is_enabled(&self, feature: &str, context: &Context) -> Option<bool> {
        match feature {
            "enabled" => Some(true),
            "disabled" => Some(false),
            "conditional" => context
                .iter()
                .find_map(|context| context.extensions().get::<Foo>())
                .map(|Foo(value)| *value),
            _ => None,
        }
    }

    fn on_new_context(&self, mut context: ContextRef<'_>, fields: Fields<'_>) {
        let foo_value = fields.get("foo").and_then(|value| value.as_bool());
        if let Some(foo_value) = foo_value {
            context.extensions_mut().insert(Foo(foo_value));
        }
    }
}

struct Foo(bool);

fn main() {
    set_global_default(SomeEvaluator::new());

    if featureflag::is_enabled!("enabled", false) {
        println!("feature \"enabled\" is enabled");
    } else {
        println!("feature \"enabled\" is not enabled");
    }

    if featureflag::is_enabled!("disabled", false) {
        println!("feature \"disabled\" is enabled");
    } else {
        println!("feature \"disabled\" is not enabled");
    }

    if featureflag::is_enabled!("unknown", false) {
        println!("feature \"unknown\" is enabled");
    } else {
        println!("feature \"unknown\" is not enabled");
    }

    if featureflag::is_enabled!("conditional", false) {
        println!("feature \"conditional\" is enabled outside of context");
    } else {
        println!("feature \"conditional\" is not enabled outside of context");
    }

    context!(foo = false).in_scope(|| {
        if featureflag::is_enabled!("conditional", false) {
            println!("feature \"conditional\" is enabled inside of non-foo context");
        } else {
            println!("feature \"conditional\" is not enabled inside of non-foo context");
        }
    });

    context!(foo = true).in_scope(|| {
        if featureflag::is_enabled!("conditional", false) {
            println!("feature \"conditional\" is enabled inside of foo context");
        } else {
            println!("feature \"conditional\" is not enabled inside of foo context");
        }

        context!().in_scope(|| {
            if featureflag::is_enabled!("conditional", false) {
                println!("feature \"conditional\" is enabled inside of nested context");
            } else {
                println!("feature \"conditional\" is not enabled inside of nested context");
            }
        });

        context!(foo = false).in_scope(|| {
            if featureflag::is_enabled!("conditional", false) {
                println!("feature \"conditional\" is enabled inside of nested non-foo context");
            } else {
                println!("feature \"conditional\" is not enabled inside of nested non-foo context");
            }
        });
    });
}
