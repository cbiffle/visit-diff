#![allow(unused)]

use diffwalk::Diff;

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
fn foo() {
}
