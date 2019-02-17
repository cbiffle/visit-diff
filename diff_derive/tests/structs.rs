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
