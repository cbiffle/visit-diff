use visit_diff::debug_diff;
use visit_diff::Diff;

#[macro_use]
mod common;

/// enum variations
#[derive(Copy, Clone, Diff, Debug)]
enum TestEnum {
    /// unit
    A,
    /// named fields
    B { unit: (), size: usize },
    /// unnamed fields
    C(bool, usize),
}

/// zero-variant enum
#[derive(Diff, Debug)]
#[allow(unused)] // just making sure it compiles
enum EnumZ {}

debug_equivalence! {
    unit => TestEnum::A;
    r#struct => TestEnum::B { unit: (), size: 12 };
    tuple => TestEnum::C(true, 42);
}

#[test]
fn unit_enum_same() {
    use visit_diff::record::*;
    let diff = record_diff(&TestEnum::A, &TestEnum::A);
    assert_eq!(diff, Value::Same("A".into(), "A".into()));
}

#[test]
fn enum_different_shape() {
    use visit_diff::record::*;
    let diff = record_diff(&TestEnum::A, &TestEnum::B { unit: (), size: 12 });
    assert_eq!(diff, Value::Difference("A".into(),
                                       "B { unit: (), size: 12 }".into()));
}

#[test]
fn enum_different_field_struct() {
    use visit_diff::record::*;
    let diff = record_diff(
        &TestEnum::B { unit: (), size: 14 },
        &TestEnum::B { unit: (), size: 12 },
    );
    assert_eq!(diff, Value::Enum(Enum {
        name: "TestEnum",
        variant: Variant::Struct(Struct {
            name: "B",
            fields: vec![
                ("unit", Some(Value::Same("()".into(), "()".into()))),
                ("size", Some(Value::Difference("14".into(), "12".into()))),
            ],
        }),
    }));
}

#[test]
fn enum_different_field_tuple() {
    use visit_diff::record::*;
    let diff = record_diff(
        &TestEnum::C(true, 14),
        &TestEnum::C(true, 12),
    );
    assert_eq!(diff, Value::Enum(Enum {
        name: "TestEnum",
        variant: Variant::Tuple(Tuple {
            name: "C",
            fields: vec![
                Some(Value::Same("true".into(), "true".into())),
                Some(Value::Difference("14".into(), "12".into())),
            ],
        }),
    }));
}

