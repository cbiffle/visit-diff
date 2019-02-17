#![allow(unused)]

use visit_diff::debug_diff;
use visit_diff::Diff;

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
enum EnumZ {}

macro_rules! debug_equivalence {
    ($($name:ident => $x:expr;)*) => {
        mod debug_equiv {
            use super::*;
            $(
                #[test]
                fn $name() {
                    let x = $x;
                    assert_eq!(format!("{:?}", x),
                               format!("{:?}", debug_diff(&x, &x)));
                }
            )*
        }
    };
}

debug_equivalence! {
    actual_unit => ();
    refs => &&mut &();
    r#struct => TestStruct { a: true, b: () };
    tuple_struct => TestTStruct(true, ());
    unit => TestUStruct;
    enum_a => TestEnum::A;
    enum_b => TestEnum::B { unit: (), size: 12 };
    enum_c => TestEnum::C(true, 42);
    slice => &[TestEnum::A, TestEnum::B { unit: (), size: 0 }] as &[TestEnum];
    vec => vec![TestEnum::A; 10];
    u32 => 42u32;
    str => "hello, world";
    cell => core::cell::Cell::new(42u32);
    ref_cell => core::cell::RefCell::new(42u32);
}
