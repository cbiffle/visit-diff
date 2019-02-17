#![allow(unused)]

use visit_diff::Diff;
use visit_diff::debug_diff;

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

/// enum variations
#[derive(Diff, Debug)]
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
enum EnumZ {}

#[test]
fn debug_struct() {
    let s = TestStruct { a: true, b: () };
    assert_eq!(format!("{:?}", s), format!("{:?}", debug_diff(&s, &s)));
}

#[test]
fn debug_enum_a() {
    let s = TestEnum::A;
    assert_eq!(format!("{:?}", s), format!("{:?}", debug_diff(&s, &s)));
}

#[test]
fn debug_enum_b() {
    let s = TestEnum::B { unit: (), size: 12 };
    assert_eq!(format!("{:?}", s), format!("{:?}", debug_diff(&s, &s)));
}

#[test]
fn debug_enum_c() {
    let s = TestEnum::C(true, 42);
    assert_eq!(format!("{:?}", s), format!("{:?}", debug_diff(&s, &s)));
}
