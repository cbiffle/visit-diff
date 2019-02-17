use visit_diff::debug_diff;
use visit_diff::Diff;

#[macro_use]
mod common;

/// structy struct
#[derive(Diff, Debug)]
struct TestStruct {
    a: bool,
    b: (),
}

/// tuple struct
#[derive(Diff, Debug)]
struct TestTStruct(bool, ());

/// unit struct
#[derive(Diff, Debug)]
struct TestUStruct;

debug_equivalence! {
    r#struct => TestStruct { a: true, b: () };
    tuple_struct => TestTStruct(true, ());
    unit_struct => TestUStruct;
}

#[test]
fn unit_struct_same() {
    use visit_diff::record::*;
    let diff = record_diff(&TestUStruct, &TestUStruct);
    assert_eq!(diff, Value::Same("TestUStruct".into(), "TestUStruct".into()));
}

#[test]
fn field_struct_same() {
    use visit_diff::record::*;
    let a = TestStruct { a: false, b: () };
    let diff = record_diff(&a, &a);
    assert_eq!(diff, Value::Struct(Struct {
        name: "TestStruct",
        fields: vec![
            ("a", Some(Value::Same("false".into(), "false".into()))),
            ("b", Some(Value::Same("()".into(), "()".into()))),
        ],
    }));
}

#[test]
fn tuple_struct_same() {
    use visit_diff::record::*;
    let a = TestTStruct(false, ());
    let diff = record_diff(&a, &a);
    assert_eq!(diff, Value::Tuple(Tuple {
        name: "TestTStruct",
        fields: vec![
            Some(Value::Same("false".into(), "false".into())),
            Some(Value::Same("()".into(), "()".into())),
        ],
    }));
}
